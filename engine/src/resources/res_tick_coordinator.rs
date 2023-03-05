use crate::action::Action;

const ACTION_QUEUE_SLOTS: usize = 128;

/// Manages when we are allowed to tick the simulation
/// and what actions are to be applied in a particular tick
pub struct TickCoordinator {
    /// The current simulation tick
    pub current_tick: usize,

    /// The maximum tick we can process currently,
    /// i.e. we should block if we're waiting for new actions
    pub max_tick: usize,

    /// Circular queue of actions, use n%128 slot for tick n and clear
    pub action_queue: [Vec<Action>; ACTION_QUEUE_SLOTS],
}

impl TickCoordinator {
    pub fn new() -> Self {
        const EMPTY_VEC: Vec<Action> = Vec::new();

        Self {
            current_tick: 0,
            // Testing
            //max_tick: 0,
            max_tick: 1 << 30,
            action_queue: [EMPTY_VEC; ACTION_QUEUE_SLOTS],
        }
    }

    /// Retrieves the queue slot for the specified tick
    fn queue_slot_at(&mut self, tick: usize) -> &mut Vec<Action> {
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

    /// Retrieves the queue slot for the current tick, shared
    pub fn current_tick_actions(&self) -> &Vec<Action> {
        &self.action_queue[self.current_tick % ACTION_QUEUE_SLOTS]
    }

    /// Retrieves the queue slot for the current tick, exclusively
    fn current_queue_slot(&mut self) -> &mut Vec<Action> {
        self.queue_slot_at(self.current_tick)
    }

    pub fn enqueue_action(&mut self, action: Action, tick: usize) {
        assert!(
            tick > self.current_tick,
            "Attempted to enqueue an action at past tick {tick}, currently at {}",
            self.current_tick
        );

        self.queue_slot_at(tick).push(action);
    }

    pub fn advance(&mut self) {
        // Should not advance past the current action horizon
        assert!(
            self.current_tick < self.max_tick,
            "Attempted to advance past the current action horizon at tick {}",
            self.current_tick
        );

        // Remove events from current queue slot
        self.current_queue_slot().clear();

        // advance the tick counter
        self.current_tick += 1;
    }
}
