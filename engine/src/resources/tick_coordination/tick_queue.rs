use crate::action::Action;

const ACTION_QUEUE_SLOTS: usize = 128;

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

    /// The queue of upcoming actions
    action_queue: [QueueSlot; ACTION_QUEUE_SLOTS],
}

impl TickQueue {
    pub fn new() -> Self {
        const EMPTY_SLOT: QueueSlot = QueueSlot {
            state: QueueSlotState::Pending,
            actions: Vec::new(),
        };

        Self {
            current_tick: 0,
            action_queue: [EMPTY_SLOT; ACTION_QUEUE_SLOTS],
        }
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
        let slot = self.queue_slot_at(tick);

        // Should not finalize an already finalized tick
        assert!(
            !slot.is_finalized(),
            "Attempted to finalize the current tick {}",
            self.current_tick
        );

        slot.state = QueueSlotState::Finalized;
    }

    pub fn finalize_tick_with_actions(&mut self, tick: usize, mut actions: Vec<Action>) {
        let slot = self.queue_slot_at(tick);

        // Should not finalize an already finalized tick
        assert!(
            !slot.is_finalized(),
            "Attempted to finalize the current tick {}",
            self.current_tick
        );

        slot.actions.append(&mut actions);
        slot.state = QueueSlotState::Finalized;
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
    }

    pub fn is_next_tick_finalized(&self) -> bool {
        self.peek_queue_slot_at(self.current_tick + 1)
            .is_finalized()
    }
}
