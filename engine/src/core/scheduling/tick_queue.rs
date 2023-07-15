use bevy::prelude::Resource;

use super::Action;
use crate::core::networking::NetworkMessage;

const ACTION_QUEUE_SLOTS: usize = 128;

enum QueueSlotState {
    /// We should not yet process this slot
    Pending,

    /// We can process this slot
    Finalized,

    /// We have simulated the actions in this slot
    Simulated,
}

struct QueueSlot {
    state: QueueSlotState,
    actions: Vec<Action>,

    // If this slot is finalized and simulated, the checksum of the state after
    checksum: u64,
}

#[allow(unused)]
impl QueueSlot {
    pub fn is_pending(&self) -> bool {
        matches!(self.state, QueueSlotState::Pending)
    }

    pub fn is_finalized(&self) -> bool {
        matches!(self.state, QueueSlotState::Finalized)
    }

    pub fn is_simulated(&self) -> bool {
        matches!(self.state, QueueSlotState::Simulated)
    }

    pub fn on_simulated(&mut self, checksum: u64) {
        assert!(self.is_finalized());
        self.state = QueueSlotState::Simulated;
        self.checksum = checksum;
    }

    pub fn reset(&mut self) {
        self.state = QueueSlotState::Pending;
        self.actions.clear();
        self.checksum = 0;
    }
}

// Special cases: Tick 0 is always considered simulated

#[derive(Resource)]
pub struct ResTickQueue {
    /// The current simulation tick
    next_tick_to_simulate: usize,

    /// The last tick that is finalized (has no non-finalized ticks before it)
    last_finalized_tick: usize,

    first_populated_slot: usize,

    /// The queue of upcoming actions
    action_queue: [QueueSlot; ACTION_QUEUE_SLOTS],
}

impl Default for ResTickQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl ResTickQueue {
    pub fn new() -> Self {
        const EMPTY_SLOT: QueueSlot = QueueSlot {
            state: QueueSlotState::Pending,
            actions: Vec::new(),
            checksum: 0,
        };

        let mut action_queue = [EMPTY_SLOT; ACTION_QUEUE_SLOTS];

        // XXX
        action_queue[0].state = QueueSlotState::Simulated;

        Self {
            next_tick_to_simulate: 1,
            last_finalized_tick: 0,
            first_populated_slot: 0,
            action_queue,
        }
    }

    pub fn get_last_simulated_tick(&self) -> usize {
        self.next_tick_to_simulate - 1
    }

    pub fn get_next_tick_to_simulate(&self) -> usize {
        self.next_tick_to_simulate
    }

    pub fn get_last_finalized_tick(&self) -> usize {
        self.last_finalized_tick
    }

    pub fn set_last_simulated_tick(&mut self, tick: usize) {
        self.last_finalized_tick = tick;
        self.next_tick_to_simulate = tick + 1;
        self.first_populated_slot = tick;

        // Reset everything, something else is responsible now
        for slot in self.action_queue.iter_mut() {
            slot.reset();
        }

        //self.queue_slot_at(self.get_last_simulated_tick()).state = QueueSlotState::Simulated;
    }

    /// Retrieves the queue slot for the specified tick
    fn queue_slot_at(&mut self, tick: usize) -> &mut QueueSlot {
        assert!(
            tick >= self.next_tick_to_simulate,
            "Attempted to retrieve action queue from the past tick {tick}, currently at {}",
            self.next_tick_to_simulate
        );

        assert!(
            tick < self.next_tick_to_simulate + ACTION_QUEUE_SLOTS,
            "Attempted to retrieve action queue too far in the future at tick {tick}, currently at {}",
            self.next_tick_to_simulate
        );

        self.inner_queue_slot_at(tick)
    }

    /// Retrieves any queue slot, regardless of current state
    fn inner_queue_slot_at(&mut self, tick: usize) -> &mut QueueSlot {
        &mut self.action_queue[tick % ACTION_QUEUE_SLOTS]
    }

    /// Retrieves the queue slot for the specified tick (shared reference)
    fn peek_queue_slot_at(&self, tick: usize) -> &QueueSlot {
        assert!(
            tick >= self.next_tick_to_simulate,
            "Attempted to retrieve action queue from the past tick {tick}, currently at {}",
            self.next_tick_to_simulate
        );

        assert!(
            tick < self.next_tick_to_simulate + ACTION_QUEUE_SLOTS,
            "Attempted to retrieve action queue too far in the future at tick {tick}, currently at {}",
            self.next_tick_to_simulate
        );

        &self.action_queue[tick % ACTION_QUEUE_SLOTS]
    }

    /// Retrieves the queue slot for the current tick, shared
    pub fn current_tick_actions(&self) -> &Vec<Action> {
        &self.action_queue[self.next_tick_to_simulate % ACTION_QUEUE_SLOTS].actions
    }

    /// Retrieves the queue slot for the latest finalized tick, shared
    pub fn last_finalized_tick_actions(&self) -> &Vec<Action> {
        &self.peek_queue_slot_at(self.last_finalized_tick).actions
    }

    /// Retrieves the queue slot for the current tick, exclusively
    fn current_queue_slot(&mut self) -> &mut QueueSlot {
        self.queue_slot_at(self.next_tick_to_simulate)
    }

    /// Enqueue an action in the next possible tick
    pub fn enqueue_action_immediately(&mut self, action: Action) {
        self.enqueue_action(action, self.last_finalized_tick + 1);
    }

    pub fn enqueue_actions_immediately(&mut self, actions: Vec<Action>) {
        self.enqueue_actions(actions, self.last_finalized_tick + 1);
    }

    pub fn enqueue_action(&mut self, action: Action, tick: usize) {
        assert!(
            tick > self.next_tick_to_simulate,
            "Attempted to enqueue an action at past tick {tick}, currently at {}",
            self.next_tick_to_simulate
        );

        self.queue_slot_at(tick).actions.push(action);
    }

    pub fn enqueue_actions(&mut self, actions: Vec<Action>, tick: usize) {
        assert!(
            tick > self.next_tick_to_simulate,
            "Attempted to enqueue an action at past tick {tick}, currently at {}",
            self.next_tick_to_simulate
        );

        self.queue_slot_at(tick).actions.extend(actions);
    }

    pub fn finalize_tick(&mut self, tick: usize) {
        let slot = self.queue_slot_at(tick);

        // Should not finalize an already finalized tick
        assert!(
            !slot.is_finalized(),
            "Attempted to finalize the current tick {}",
            self.next_tick_to_simulate
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

    pub fn reset_simulated(&mut self) {
        let last_sim = self.get_last_simulated_tick();
        self.reset_through(last_sim);
    }

    pub fn reset_through(&mut self, tick: usize) {
        assert!(
            tick < self.get_next_tick_to_simulate(),
            "Attempted to reset into currently simulated ticks",
        );

        // Reset everything, something else is responsible now
        for slot in self.first_populated_slot..tick {
            self.inner_queue_slot_at(slot).reset();
        }

        self.first_populated_slot = tick;
    }

    /// Call this when the current tick has been simulated
    pub fn advance(&mut self, checksum: u64) {
        // Remove events from current queue slot
        //self.current_queue_slot().reset();
        self.current_queue_slot().on_simulated(checksum);

        // advance the tick counter
        self.next_tick_to_simulate += 1;

        // Should not advance past the current finalization horizon
        // assert!(
        //     self.current_queue_slot().is_finalized(),
        //     "Attempted to advance past the current action horizon at tick {}",
        //     self.next_tick_to_simulate
        // );

        assert!(
            self.get_last_simulated_tick() <= self.last_finalized_tick,
            "Attempted to advance past the finalized tick counter: current {} > finalized {}",
            self.next_tick_to_simulate,
            self.last_finalized_tick
        );
    }

    pub fn is_next_tick_finalized(&self) -> bool {
        self.peek_queue_slot_at(self.next_tick_to_simulate)
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
            assert!(
                slot.is_finalized(),
                "Slot at tick {} is not finalized",
                tick
            );

            messages.push(NetworkMessage::FinalizedTick {
                tick,
                actions: slot.actions.clone(),
            });
        }

        (self.last_finalized_tick, messages)
    }
}

#[test]
fn basic_test() {
    use crate::core::scheduling::Direction;

    let mut queue = ResTickQueue::new();

    assert_eq!(
        queue.next_tick_to_simulate, 0,
        "Tick queue should start at tick 0"
    );

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
    queue.advance(0);
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
    queue.advance(0);
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
    queue.advance(0);
    let current_actions = queue.current_tick_actions();

    assert_eq!(
        current_actions.len(),
        0,
        "Tick queue should have no actions after advancing to tick 3"
    );
}
