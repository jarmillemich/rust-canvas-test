use super::types::NetworkMessage;
use crate::action::Action;

// XXX cheating
const ACTION_QUEUE_SLOTS: usize = 1024;

enum QueueSlotState {
    /// We should not yet process this slot
    Pending,

    /// We can process this slot
    Finalized,
}

struct QueueSlot {
    state: QueueSlotState,
    actions: Vec<Action>,
}

#[allow(unused)]
impl QueueSlot {
    pub fn is_pending(&self) -> bool {
        matches!(self.state, QueueSlotState::Pending)
    }

    pub fn is_finalized(&self) -> bool {
        matches!(self.state, QueueSlotState::Finalized)
    }

    pub fn reset(&mut self) {
        self.state = QueueSlotState::Pending;
        self.actions.clear();
    }
}

pub struct TickQueue {
    /// The current simulation tick
    pub current_tick: usize,

    /// The last tick that is finalized (has no non-finalized ticks before it)
    last_finalized_tick: usize,

    /// The queue of upcoming actions
    action_queue: [QueueSlot; ACTION_QUEUE_SLOTS],
}

impl Default for TickQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl TickQueue {
    pub fn new() -> Self {
        const EMPTY_SLOT: QueueSlot = QueueSlot {
            state: QueueSlotState::Pending,
            actions: Vec::new(),
        };

        let mut action_queue = [EMPTY_SLOT; ACTION_QUEUE_SLOTS];
        // Mark the zeroth tick as always finalized to get us started
        action_queue[0].state = QueueSlotState::Finalized;

        Self {
            current_tick: 0,
            last_finalized_tick: 0,
            action_queue,
        }
    }

    pub fn get_last_finalized_tick(&self) -> usize {
        self.last_finalized_tick
    }

    pub fn set_last_finalized_tick(&mut self, tick: usize) {
        self.last_finalized_tick = tick;
        self.current_tick = tick;

        // Reset everything, something else is responsible now
        // for slot in self.action_queue.iter_mut() {
        //     slot.reset();
        // }

        self.finalize_tick(tick);
    }

    /// Retrieves the queue slot for the specified tick
    fn queue_slot_at(&mut self, tick: usize) -> &mut QueueSlot {
        assert!(
            tick >= self.current_tick,
            "Attempted to retrieve action queue from the past tick {tick}, currently at {}",
            self.current_tick
        );

        assert!(
            tick < self.current_tick + ACTION_QUEUE_SLOTS,
            "Attempted to retrieve action queue too far in the future at tick {tick}, currently at {}",
            self.current_tick
        );

        &mut self.action_queue[tick % ACTION_QUEUE_SLOTS]
    }

    /// Retrieves the queue slot for the specified tick (shared reference)
    fn peek_queue_slot_at(&self, tick: usize) -> &QueueSlot {
        assert!(
            tick >= self.current_tick,
            "Attempted to retrieve action queue from the past tick {tick}, currently at {}",
            self.current_tick
        );

        assert!(
            tick < self.current_tick + ACTION_QUEUE_SLOTS,
            "Attempted to retrieve action queue too far in the future at tick {tick}, currently at {}",
            self.current_tick
        );

        &self.action_queue[tick % ACTION_QUEUE_SLOTS]
    }

    /// Retrieves the queue slot for the current tick, shared
    pub fn current_tick_actions(&self) -> &Vec<Action> {
        &self.action_queue[self.current_tick % ACTION_QUEUE_SLOTS].actions
    }

    /// Retrieves the queue slot for the latest finalized tick, shared
    pub fn last_finalized_tick_actions(&self) -> &Vec<Action> {
        &self.peek_queue_slot_at(self.last_finalized_tick).actions
    }

    /// Retrieves the queue slot for the current tick, exclusively
    fn current_queue_slot(&mut self) -> &mut QueueSlot {
        self.queue_slot_at(self.current_tick)
    }

    pub fn enqueue_action(&mut self, action: Action, tick: usize) {
        assert!(
            tick > self.current_tick,
            "Attempted to enqueue an action at past tick {tick}, currently at {}",
            self.current_tick
        );

        self.queue_slot_at(tick).actions.push(action);
    }

    pub fn finalize_tick(&mut self, tick: usize) {
        // web_sys::console::log_1(
        //     &format!(
        //         "Finalizing tick {}, last finalized {}",
        //         tick, self.last_finalized_tick
        //     )
        //     .into(),
        // );

        let slot = self.queue_slot_at(tick);

        // Should not finalize an already finalized tick
        assert!(
            !slot.is_finalized(),
            "Attempted to finalize the current tick {}",
            self.current_tick
        );

        slot.state = QueueSlotState::Finalized;

        // Move up our last finalized tick counter
        while self
            .peek_queue_slot_at(self.last_finalized_tick + 1)
            .is_finalized()
        {
            self.last_finalized_tick += 1;
        }
    }

    pub fn finalize_tick_with_actions(&mut self, tick: usize, mut actions: Vec<Action>) {
        let slot = self.queue_slot_at(tick);

        // Should not finalize an already finalized tick
        assert!(
            !slot.is_finalized(),
            "Attempted to finalize an already finalized tick tick {}",
            tick
        );

        slot.actions.append(&mut actions);

        self.finalize_tick(tick);
    }

    pub fn advance(&mut self) {
        // Remove events from current queue slot
        self.current_queue_slot().reset();

        // advance the tick counter
        self.current_tick += 1;

        // Should not advance past the current finalization horizon
        assert!(
            self.current_queue_slot().is_finalized(),
            "Attempted to advance past the current action horizon at tick {}",
            self.current_tick
        );

        assert!(
            self.current_tick <= self.last_finalized_tick,
            "Attempted to advance past the finalized tick counter: current {} > finalized {}",
            self.current_tick,
            self.last_finalized_tick
        );
    }

    pub fn is_next_tick_finalized(&self) -> bool {
        self.peek_queue_slot_at(self.current_tick + 1)
            .is_finalized()
    }

    pub fn next_unfinalized_tick(&self) -> usize {
        assert!(
            !self
                .peek_queue_slot_at(self.last_finalized_tick + 1)
                .is_finalized(),
            "Tick slot after last finalized should not be finalized, at {}",
            self.last_finalized_tick
        );

        self.last_finalized_tick + 1
    }

    pub fn make_tick_finalization_messages(
        &self,
        from_tick: usize,
    ) -> (usize, Vec<NetworkMessage>) {
        let mut messages = Vec::new();

        for tick in from_tick..=self.last_finalized_tick {
            let slot = self.peek_queue_slot_at(tick);

            if slot.is_finalized() {
                messages.push(NetworkMessage::FinalizedTick {
                    tick,
                    actions: slot.actions.clone(),
                });
            }
        }

        (self.last_finalized_tick, messages)
    }
}

#[test]
fn basic_test() {
    use crate::action::Direction;

    let mut queue = TickQueue::new();

    assert_eq!(queue.current_tick, 0, "Tick queue should start at tick 0");

    assert_eq!(
        queue.current_tick_actions().len(),
        0,
        "Tick queue should start with no actions"
    );

    // Queue up an action in the next tick
    queue.finalize_tick_with_actions(1, vec![Action::Jump]);

    // Queue up a couple actions in the subsequent tick
    queue.finalize_tick_with_actions(
        2,
        vec![
            Action::Fire,
            Action::StartMoving {
                dir: Direction::Right,
            },
        ],
    );

    // And finalize one last empty tick
    queue.finalize_tick(3);

    // Advance and check that the action is available
    queue.advance();
    let current_actions = queue.current_tick_actions();

    assert_eq!(
        current_actions.len(),
        1,
        "Tick queue should have 1 action after advancing to tick 1"
    );

    assert_eq!(
        current_actions[0],
        Action::Jump,
        "Tick queue should have a jump action after advancing to tick 1"
    );

    // Advance to the next tick
    // TODO actions are technically not necessarily ordered
    queue.advance();
    let current_actions = queue.current_tick_actions();

    assert_eq!(
        current_actions.len(),
        2,
        "Tick queue should have 2 actions after advancing to tick 2"
    );

    assert_eq!(
        current_actions[0],
        Action::Fire,
        "Tick queue should have a fire action after advancing to tick 2"
    );

    assert_eq!(
        current_actions[1],
        Action::StartMoving {
            dir: Direction::Right
        },
        "Tick queue should have a start moving action after advancing to tick 2"
    );

    // Advance to the next tick (should be empty)
    queue.advance();
    let current_actions = queue.current_tick_actions();

    assert_eq!(
        current_actions.len(),
        0,
        "Tick queue should have no actions after advancing to tick 3"
    );
}
