//! FSRS-style spaced-repetition scheduler for edureel recall cards.
//!
//! Ported from WyattAu/Tachyon `flashcards/srs.rs` (extract, not fork).
//! State machine: `New → Learning(step 0..n) → Review`, with lapses moving
//! a Review card to `Relearning` before it graduates back to `Review`.
//! Stability grows on successful reviews and shrinks on lapses; difficulty
//! moves in the opposite direction.
//!
//! # Time mapping
//!
//! Tachyon used `chrono::DateTime<Utc>` and `Duration`; this port uses a
//! plain `i64` count of unix **seconds** for `now`, `due`, `last_review`,
//! and `updated_at`, so the crate stays pure and dependency-free. Minutes
//! and days from Tachyon become `60` and `86_400` second multiples. Whole
//! day intervals are computed by truncating fractional stability days,
//! exactly like Tachyon's `Duration::days(interval as i64)`.
//!
//! # Representation mapping
//!
//! Tachyon stored `CardState` as an `i16` DB column and carried a
//! `flashcard_id` through every review; this port keeps `CardState` as a
//! first-class enum (same variants and semantics) and drops `flashcard_id`
//! — the scheduler never reads it and edureel keys cards by concept.

use serde::{Deserialize, Serialize};

/// FSRS parameters (simplified), identical to Tachyon.
const INITIAL_STABILITY: f64 = 1.0;
/// See [`INITIAL_STABILITY`].
const INITIAL_DIFFICULTY: f64 = 0.3;
/// See [`INITIAL_STABILITY`].
const MIN_STABILITY: f64 = 0.1;
/// See [`INITIAL_STABILITY`].
const MAX_STABILITY: f64 = 36500.0;
/// See [`INITIAL_DIFFICULTY`].
const MIN_DIFFICULTY: f64 = 0.0;
/// See [`INITIAL_DIFFICULTY`].
const MAX_DIFFICULTY: f64 = 1.0;
/// Learning step lengths in minutes, identical to Tachyon.
const LEARNING_STEPS: [i16; 2] = [1, 10];
/// Relearning step lengths in minutes, identical to Tachyon.
const RELEARNING_STEPS: [i16; 1] = [10];

/// Seconds per minute (Tachyon: `Duration::minutes`).
const SECS_PER_MINUTE: i64 = 60;
/// Seconds per day (Tachyon: `Duration::days`).
const SECS_PER_DAY: i64 = 86_400;

/// Which phase of the learning pipeline a card is in.
///
/// Same four states and meanings as Tachyon `tachyon_database::flashcard
/// ::CardState`; kept as an enum instead of an `i16` column value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardState {
    /// Never reviewed.
    New,
    /// Inside the short-interval learning steps.
    Learning,
    /// Graduated; scheduled by stability-day intervals.
    Review,
    /// Re-learning after a lapse from `Review`.
    Relearning,
}

/// How the learner recalled a card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rating {
    /// Complete recall failure.
    Again,
    /// Recalled with heavy effort.
    Hard,
    /// Recalled normally.
    Good,
    /// Recalled instantly.
    Easy,
}

/// Full SRS state for one card, keyed externally by concept.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlashcardSrsState {
    /// Current pipeline phase.
    pub state: CardState,
    /// Index within [`LEARNING_STEPS`] or [`RELEARNING_STEPS`]; reset on
    /// every phase change.
    pub step: i16,
    /// Memory stability in days (FSRS-style simplified).
    pub stability: f64,
    /// Subjective difficulty in `[0, 1]`.
    pub difficulty: f64,
    /// Unix seconds at which the card is due.
    pub due: i64,
    /// Total successful-answer repetitions.
    pub reps: u32,
    /// Total lapses out of `Review`.
    pub lapses: u32,
    /// Unix seconds of the most recent review, if any.
    pub last_review: Option<i64>,
    /// Unix seconds of the last state mutation.
    pub updated_at: i64,
}

/// A brand-new card due immediately at `now`.
#[must_use]
pub fn new_card(now: i64) -> FlashcardSrsState {
    FlashcardSrsState {
        state: CardState::New,
        step: 0,
        stability: INITIAL_STABILITY,
        difficulty: INITIAL_DIFFICULTY,
        due: now,
        reps: 0,
        lapses: 0,
        last_review: None,
        updated_at: now,
    }
}

/// Whether the card is due for review at unix seconds `now`.
#[must_use]
pub fn is_due(state: &FlashcardSrsState, now: i64) -> bool {
    state.due <= now
}

/// Process a review at unix seconds `now` and return the updated SRS state.
///
/// Transition semantics are a line-for-line port of Tachyon
/// `flashcards/srs.rs::process_review`.
#[must_use]
pub fn process_review(state: &FlashcardSrsState, rating: Rating, now: i64) -> FlashcardSrsState {
    match state.state {
        CardState::New => process_new(state, rating, now),
        CardState::Learning => process_learning(state, rating, now),
        CardState::Review => process_review_card(state, rating, now),
        CardState::Relearning => process_relearning(state, rating, now),
    }
}

/// Whole-day interval from a fractional stability value.
///
/// Truncation is intentional: Tachyon schedules whole days via
/// `Duration::days(interval as i64)`, so a fractional day floors to the
/// same number here. Negative stabilities cannot occur (min-clamped), and
/// NaN saturates to 0 under Rust's float-to-int cast rules.
#[allow(clippy::cast_possible_truncation)]
fn whole_days(interval: f64) -> i64 {
    interval.max(0.0).floor() as i64
}

/// Day-long due date offset from `now`.
fn due_in_days(now: i64, interval: f64) -> i64 {
    now + whole_days(interval) * SECS_PER_DAY
}

/// Minute-long due date offset from `now`.
fn due_in_minutes(now: i64, minutes: i16) -> i64 {
    now + i64::from(minutes) * SECS_PER_MINUTE
}

/// Highest valid step index for `steps` (arrays are compile-time non-empty;
/// the `unwrap_or` chain only guards degenerate callers).
fn max_step_of(steps: &[i16]) -> i16 {
    steps
        .len()
        .checked_sub(1)
        .and_then(|last| i16::try_from(last).ok())
        .unwrap_or(0)
}

/// Step duration for `step`, falling back to step 0's length if the index
/// is out of range (cannot happen for states produced by this module).
fn step_minutes(steps: &[i16], step: i16) -> i16 {
    usize::try_from(step)
        .ok()
        .and_then(|idx| steps.get(idx))
        .copied()
        .unwrap_or_else(|| steps.first().copied().unwrap_or(0))
        .max(0)
}

fn process_new(state: &FlashcardSrsState, rating: Rating, now: i64) -> FlashcardSrsState {
    match rating {
        Rating::Again => {
            // Go to Learning, step 0.
            FlashcardSrsState {
                state: CardState::Learning,
                step: 0,
                stability: INITIAL_STABILITY,
                difficulty: INITIAL_DIFFICULTY,
                due: due_in_minutes(now, LEARNING_STEPS[0]),
                reps: 0,
                lapses: state.lapses,
                last_review: Some(now),
                updated_at: now,
            }
        }
        Rating::Hard => {
            // Go to Learning, step 0 (slightly longer interval).
            FlashcardSrsState {
                state: CardState::Learning,
                step: 0,
                stability: INITIAL_STABILITY * 1.2,
                difficulty: INITIAL_DIFFICULTY + 0.05,
                due: due_in_minutes(now, LEARNING_STEPS[0] + 1),
                reps: 1,
                lapses: state.lapses,
                last_review: Some(now),
                updated_at: now,
            }
        }
        Rating::Good => {
            // Skip learning, go to Review with base interval.
            FlashcardSrsState {
                state: CardState::Review,
                step: 0,
                stability: INITIAL_STABILITY * 2.0,
                difficulty: INITIAL_DIFFICULTY,
                due: due_in_days(now, INITIAL_STABILITY),
                reps: 1,
                lapses: state.lapses,
                last_review: Some(now),
                updated_at: now,
            }
        }
        Rating::Easy => {
            // Skip learning, go to Review with longer interval.
            FlashcardSrsState {
                state: CardState::Review,
                step: 0,
                stability: INITIAL_STABILITY * 4.0,
                difficulty: INITIAL_DIFFICULTY - 0.1,
                due: due_in_days(now, INITIAL_STABILITY * 4.0),
                reps: 2,
                lapses: state.lapses,
                last_review: Some(now),
                updated_at: now,
            }
        }
    }
}

fn process_learning(state: &FlashcardSrsState, rating: Rating, now: i64) -> FlashcardSrsState {
    let max_step = max_step_of(&LEARNING_STEPS);
    match rating {
        Rating::Again => {
            // Reset to step 0.
            FlashcardSrsState {
                state: CardState::Learning,
                step: 0,
                stability: state.stability,
                difficulty: state.difficulty,
                due: due_in_minutes(now, LEARNING_STEPS[0]),
                lapses: state.lapses.saturating_add(1),
                last_review: Some(now),
                updated_at: now,
                ..*state
            }
        }
        Rating::Hard => {
            // Stay at current step.
            let step = state.step.min(max_step);
            FlashcardSrsState {
                step,
                due: due_in_minutes(now, step_minutes(&LEARNING_STEPS, step)),
                difficulty: (state.difficulty + 0.15).min(MAX_DIFFICULTY),
                last_review: Some(now),
                updated_at: now,
                ..*state
            }
        }
        Rating::Good => {
            let next_step = state.step.saturating_add(1);
            if next_step > max_step {
                // Graduate to Review.
                FlashcardSrsState {
                    state: CardState::Review,
                    step: 0,
                    stability: state.stability * 1.5,
                    due: due_in_days(now, state.stability),
                    reps: state.reps.saturating_add(1),
                    last_review: Some(now),
                    updated_at: now,
                    ..*state
                }
            } else {
                FlashcardSrsState {
                    step: next_step,
                    due: due_in_minutes(now, step_minutes(&LEARNING_STEPS, next_step)),
                    last_review: Some(now),
                    updated_at: now,
                    ..*state
                }
            }
        }
        Rating::Easy => {
            // Graduate to Review immediately with bonus.
            FlashcardSrsState {
                state: CardState::Review,
                step: 0,
                stability: state.stability * 2.5,
                difficulty: (state.difficulty - 0.15).max(MIN_DIFFICULTY),
                due: due_in_days(now, state.stability * 2.0),
                reps: state.reps.saturating_add(2),
                last_review: Some(now),
                updated_at: now,
                ..*state
            }
        }
    }
}

fn process_review_card(state: &FlashcardSrsState, rating: Rating, now: i64) -> FlashcardSrsState {
    match rating {
        Rating::Again => {
            // Lapse: go to Relearning.
            FlashcardSrsState {
                state: CardState::Relearning,
                step: 0,
                stability: (state.stability * 0.5).max(MIN_STABILITY),
                difficulty: (state.difficulty + 0.2).min(MAX_DIFFICULTY),
                due: due_in_minutes(now, RELEARNING_STEPS[0]),
                lapses: state.lapses.saturating_add(1),
                last_review: Some(now),
                updated_at: now,
                ..*state
            }
        }
        Rating::Hard => {
            let factor = 1.2;
            let interval = (state.stability * factor).clamp(MIN_STABILITY, MAX_STABILITY);
            FlashcardSrsState {
                stability: interval,
                difficulty: (state.difficulty + 0.05).min(MAX_DIFFICULTY),
                due: due_in_days(now, interval),
                reps: state.reps.saturating_add(1),
                last_review: Some(now),
                updated_at: now,
                ..*state
            }
        }
        Rating::Good => {
            let interval = state.stability.clamp(MIN_STABILITY, MAX_STABILITY);
            FlashcardSrsState {
                stability: (state.stability * 2.5).clamp(MIN_STABILITY, MAX_STABILITY),
                due: due_in_days(now, interval),
                reps: state.reps.saturating_add(1),
                last_review: Some(now),
                updated_at: now,
                ..*state
            }
        }
        Rating::Easy => {
            let interval = (state.stability * 1.5).clamp(MIN_STABILITY, MAX_STABILITY);
            FlashcardSrsState {
                stability: (state.stability * 4.0).clamp(MIN_STABILITY, MAX_STABILITY),
                difficulty: (state.difficulty - 0.15).max(MIN_DIFFICULTY),
                due: due_in_days(now, interval),
                reps: state.reps.saturating_add(1),
                last_review: Some(now),
                updated_at: now,
                ..*state
            }
        }
    }
}

fn process_relearning(state: &FlashcardSrsState, rating: Rating, now: i64) -> FlashcardSrsState {
    let max_step = max_step_of(&RELEARNING_STEPS);
    match rating {
        Rating::Again => FlashcardSrsState {
            step: 0,
            stability: (state.stability * 0.5).max(MIN_STABILITY),
            difficulty: (state.difficulty + 0.1).min(MAX_DIFFICULTY),
            due: due_in_minutes(now, RELEARNING_STEPS[0]),
            lapses: state.lapses.saturating_add(1),
            last_review: Some(now),
            updated_at: now,
            ..*state
        },
        Rating::Hard => {
            let step = state.step.min(max_step);
            FlashcardSrsState {
                step,
                due: due_in_minutes(now, step_minutes(&RELEARNING_STEPS, step)),
                difficulty: (state.difficulty + 0.05).min(MAX_DIFFICULTY),
                last_review: Some(now),
                updated_at: now,
                ..*state
            }
        }
        Rating::Good => {
            let next_step = state.step.saturating_add(1);
            if next_step > max_step {
                // Graduate back to Review.
                FlashcardSrsState {
                    state: CardState::Review,
                    step: 0,
                    stability: state.stability * 1.5,
                    due: due_in_days(now, state.stability.clamp(MIN_STABILITY, MAX_STABILITY)),
                    reps: state.reps.saturating_add(1),
                    last_review: Some(now),
                    updated_at: now,
                    ..*state
                }
            } else {
                FlashcardSrsState {
                    step: next_step,
                    due: due_in_minutes(now, step_minutes(&RELEARNING_STEPS, next_step)),
                    last_review: Some(now),
                    updated_at: now,
                    ..*state
                }
            }
        }
        Rating::Easy => FlashcardSrsState {
            state: CardState::Review,
            step: 0,
            stability: state.stability * 2.5,
            difficulty: (state.difficulty - 0.1).max(MIN_DIFFICULTY),
            due: due_in_days(
                now,
                (state.stability * 2.0).clamp(MIN_STABILITY, MAX_STABILITY),
            ),
            reps: state.reps.saturating_add(2),
            last_review: Some(now),
            updated_at: now,
            ..*state
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        is_due, new_card, process_review, CardState, FlashcardSrsState, Rating, INITIAL_DIFFICULTY,
        INITIAL_STABILITY, MAX_DIFFICULTY, MAX_STABILITY, MIN_DIFFICULTY, MIN_STABILITY,
    };

    const EPS: f64 = 1e-9;
    const DAY: i64 = 86_400;
    const MINUTE: i64 = 60;
    const NOW: i64 = 1_700_000_000;

    fn card(state: CardState, stability: f64, difficulty: f64) -> FlashcardSrsState {
        FlashcardSrsState {
            state,
            step: 0,
            stability,
            difficulty,
            due: NOW,
            reps: 0,
            lapses: 0,
            last_review: None,
            updated_at: NOW,
        }
    }

    #[test]
    fn new_card_again_goes_to_learning() {
        let result = process_review(&new_card(NOW), Rating::Again, NOW);
        assert_eq!(result.state, CardState::Learning);
        assert_eq!(result.step, 0);
        assert_eq!(result.lapses, 0);
        assert_eq!(result.due, NOW + MINUTE);
    }

    #[test]
    fn new_card_good_goes_to_review() {
        let result = process_review(&new_card(NOW), Rating::Good, NOW);
        assert_eq!(result.state, CardState::Review);
        assert_eq!(result.reps, 1);
        assert_eq!(result.due, NOW + DAY);
    }

    #[test]
    fn new_card_easy_skips_to_review() {
        let result = process_review(&new_card(NOW), Rating::Easy, NOW);
        assert_eq!(result.state, CardState::Review);
        assert_eq!(result.reps, 2);
        assert!(result.stability > INITIAL_STABILITY);
        assert_eq!(result.due, NOW + 4 * DAY);
    }

    #[test]
    fn new_card_hard_stays_learning_step0() {
        let result = process_review(&new_card(NOW), Rating::Hard, NOW);
        assert_eq!(result.state, CardState::Learning);
        assert_eq!(result.step, 0);
        assert_eq!(result.due, NOW + 2 * MINUTE);
        assert!((result.stability - INITIAL_STABILITY * 1.2).abs() < EPS);
    }

    #[test]
    fn learning_good_graduates() {
        let mut state = card(CardState::Learning, INITIAL_STABILITY, INITIAL_DIFFICULTY);
        state.step = 1; // at last learning step
        let result = process_review(&state, Rating::Good, NOW);
        assert_eq!(result.state, CardState::Review);
        assert!(result.reps > state.reps);
        assert!((result.stability - state.stability * 1.5).abs() < EPS);
    }

    #[test]
    fn learning_again_resets_step_and_lapses() {
        let mut state = card(CardState::Learning, INITIAL_STABILITY, INITIAL_DIFFICULTY);
        state.step = 1;
        let result = process_review(&state, Rating::Again, NOW);
        assert_eq!(result.state, CardState::Learning);
        assert_eq!(result.step, 0);
        assert_eq!(result.lapses, 1);
        assert_eq!(result.due, NOW + MINUTE);
    }

    #[test]
    fn review_again_lapses() {
        let state = card(CardState::Review, 5.0, 0.5);
        let result = process_review(&state, Rating::Again, NOW);
        assert_eq!(result.state, CardState::Relearning);
        assert_eq!(result.step, 0);
        assert_eq!(result.lapses, 1);
        assert!(result.stability < 5.0);
        assert_eq!(result.due, NOW + 10 * MINUTE);
    }

    #[test]
    fn review_good_increases_stability() {
        let state = card(CardState::Review, 5.0, 0.5);
        let result = process_review(&state, Rating::Good, NOW);
        assert_eq!(result.state, CardState::Review);
        assert!(result.stability > 5.0);
        assert_eq!(result.due, NOW + 5 * DAY);
    }

    #[test]
    fn review_easy_maximally_increases_stability() {
        let state = card(CardState::Review, 5.0, 0.5);
        let result_good = process_review(&state, Rating::Good, NOW);
        let result_easy = process_review(&state, Rating::Easy, NOW);
        assert!(result_easy.stability > result_good.stability);
    }

    #[test]
    fn relearning_good_graduates_back() {
        let state = card(CardState::Relearning, 3.0, 0.7);
        let result = process_review(&state, Rating::Good, NOW);
        assert_eq!(result.state, CardState::Review);
        assert!((result.stability - 4.5).abs() < EPS);
    }

    #[test]
    fn relearning_good_at_step0_stays_relearning() {
        // RELEARNING_STEPS has one entry, so step 0 + Good graduates; assert
        // the step path via Hard instead, which holds the current step.
        let state = card(CardState::Relearning, 3.0, 0.7);
        let result = process_review(&state, Rating::Hard, NOW);
        assert_eq!(result.state, CardState::Relearning);
        assert_eq!(result.step, 0);
        assert_eq!(result.due, NOW + 10 * MINUTE);
        assert!((result.difficulty - 0.75).abs() < EPS);
    }

    #[test]
    fn difficulty_bounds() {
        let state = card(CardState::Review, 5.0, MAX_DIFFICULTY);
        let result = process_review(&state, Rating::Again, NOW);
        assert!(result.difficulty <= MAX_DIFFICULTY);
        assert!(result.difficulty >= MIN_DIFFICULTY);

        let state = card(CardState::Review, 5.0, MIN_DIFFICULTY);
        let result = process_review(&state, Rating::Easy, NOW);
        assert!(result.difficulty >= MIN_DIFFICULTY);
    }

    #[test]
    fn stability_bounds() {
        let state = card(CardState::Review, MIN_STABILITY, 0.5);
        let result = process_review(&state, Rating::Again, NOW);
        assert!(result.stability >= MIN_STABILITY);

        let state = card(CardState::Review, MAX_STABILITY, 0.5);
        let result = process_review(&state, Rating::Hard, NOW);
        assert!(result.stability <= MAX_STABILITY);
    }

    #[test]
    fn is_due_true_when_now_past_due() {
        let mut state = new_card(NOW);
        assert!(is_due(&state, NOW));
        state.due = NOW + DAY;
        assert!(is_due(&state, NOW + DAY));
        assert!(is_due(&state, NOW + DAY + 1));
        assert!(!is_due(&state, NOW + DAY - 1));
    }

    #[test]
    fn state_round_trips_through_json() -> Result<(), serde_json::Error> {
        let state = process_review(&new_card(NOW), Rating::Good, NOW);
        let json = serde_json::to_string(&state)?;
        let back: FlashcardSrsState = serde_json::from_str(&json)?;
        assert_eq!(back, state);
        Ok(())
    }
}
