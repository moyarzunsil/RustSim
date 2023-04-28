#![feature(generators)] 

// Structs from the standard library
use std::{rc::Rc, cell::Cell, time::Duration};

// Import from the library the necessary structs to execute a simulation    
use rustsim::{Key, Simulation, GenBoxed, Action, State, StateKey};

// A simple model of entities A and B
// 1.- Entity B will start the simulation by doing a Passivate
// 2.- Entity A will do a Hold of 5 seconds and then check if Entity B is in Passivate
// 3.- If it's in Passivate it will emit an Activate with the Key of Entity B
// 4.- Independent of the condition in (2) it will do a Passivate
// 5.- Repeat step (2) after being Activated
// Excepting step (1) you can replace Entity A with Entity B and vice-versa
// Meaning both will do steps 2, 3 and 4 with the other entity until the simulation ends. 
fn main() {
    // Create the simulation, the type parameter () indicates that to resume the generators a value of that type has to be provided
    // Because this model doesn't require the generators to be resumed with a meaninful type the empty tuple (a.k.a the unit type) is provided
    let mut simulation: Simulation<()> = Simulation::default();
    
    // Get access to the shared state managed by the simulation
    let shared_state = simulation.state();
    
    // Temporarily extract the state leaving a default state in place
    // Without this step no modifying of the state is possible
    let mut state = shared_state.take();

    // To avoid a circular dependency problem we will use the simulation state to provide the Key for Entity B
    // First a null value is inserted in the state representing the soon to be added Key for Entity B
    // This key will be given to Entity A with an access to the state.
    // And after Entity B is inserted in the simulation this null will be replaced with the actual value for Entity B Key.
    let entity_b_key = state.insert(None);

    // A struct to keep track of whetever Entity A or Entity B are currently doing a Passivate.
    let entity_states = state.insert(Passivated { entity_a: false, entity_b: false });

    // Instantiate and insert the generators to the simulation.
    let a_key = simulation.add_generator(entity_a(Rc::clone(&shared_state), entity_b_key, entity_states));
    let b_key = simulation.add_generator(entity_b(Rc::clone(&shared_state), a_key, entity_states));
    
    // Replace the null value with Entity B Key's
    *state.get_mut(entity_b_key).unwrap() = Some(b_key);

    // Return the state back to the simulation
    // This step must be done before invoking any function that ends up advancing the simulation.
    // functions like: step_with, step, run_with_limit, run_until_empty.
    // Or the generator will end up extracting an empty state
    shared_state.set(state);

    // Schedule the entities using their associated Keys at the current simulated time (0 seconds).
    simulation.schedule_now(b_key);
    simulation.schedule_now(a_key);
    
    // Advance the simulation until a maximum of 60 simulated seconds or no more events are in the scheduler (not possible with this model)
    simulation.run_with_limit(Duration::from_secs(60));
}

// A function that will create an instance of Entity A
// It requires:
//  - shared_state: The state provided by the simulation, reference counted with interior mutability to bypass some of rust limitations around shared data.
//  - entity_b_key: To create Entity A a Key of Entity B is needed but to create Entity B a Key of entity A is needed.
//                  To solve this, a null was inserted in the simulation state and the state key given to Entity A.
//                  Entity B was given the value of Entity A Key, then using the entity_b_key (it's a copy type so it can be passed around)
//                  The null value was replaced with the actual value of Entity B Key.
//  - entity_states_key:  A state key to the struct responsible to keep track of each Entity Passive state
//                        Each entity will indicate the other it's current state using this struct as a medium.
// 
// A short explanation of both entities are explained above main but a line by line explanation is also included in the body of this function.
fn entity_a(shared_state: Rc<Cell<State>>, entity_b_key: StateKey<Option<Key>>, entity_states_key: StateKey<Passivated>) -> GenBoxed<()> {
    Box::new(move |_|{
        // Temporarily extract the state leaving a default one in place
        let mut state = shared_state.take();

        // Permanently remove from the state the value associated with entity_b_key
        // Extracting a value from the state can fail if the value was already extracted
        // And because we inserted a possible null value rust requires to check first before using the value
        // flatten will convert our Option<Option<Key>> into Option<Key> and unwrap will transform the Option<Key> into Key
        // unwrap will exit the program if the value is not present.
        let entity_b_key = state.remove(entity_b_key).flatten().unwrap();

        // Return the extracted state back to the simulation
        // WARNING: This has to be done before a yield point or all other generators will recieve the default state without any value inside
        shared_state.set(state);

        loop {
            // Emit a Hold event with 5 seconds duration.
            // You could change this to be a random number
            println!("[ENTITY A] -> HOLD");
            yield Action::Hold(Duration::from_secs(5));
            println!("[ENTITY A] <- HOLD");

            let mut state = shared_state.take();

            // Get a mutable borrow of the Passivated struct 
            let entity_states = state.get_mut(entity_states_key).unwrap();

            // If entity_b is in passivate
            if entity_states.entity_b {
                // Return the state before a yield
                shared_state.set(state);

                // Emit the Activate event
                println!("[ENTITY A] -> ACTIVATE [ENTITY B]");
                yield Action::ActivateOne(entity_b_key);
            } else {
                // If it is not in Passivate, we must still return the state back to the simulation in the else branch
                // Otherwise we are left in an inconsistent state (in fact the code does not compile without this)
                // Because we would on one branch return state back but on another we don't which Rust rejects.
                shared_state.set(state);
            }
            // After all that take back the state
            let mut state = shared_state.take();

            // Modify our state to indicate that it's doing a passivate
            state.get_mut(entity_states_key).unwrap().entity_a = true;

            // Return the state before a yield
            shared_state.set(state);

            // Emit the Passivate event
            println!("[ENTITY A] -> PASSIVATE");
            yield Action::Passivate;
            println!("[ENTITY A] <- PASSIVATE");

            // After the yield means that the passivate ended so we have to modify back our state
            let mut state = shared_state.take();

            // Indicate that we are no longer doing a passivate
            state.get_mut(entity_states_key).unwrap().entity_a = false;

            // Return the state before doing a yield
            shared_state.set(state);
        }
    })
}

// A function that will create an instance of Entity B
// It's almost the same as Entity A with the difference that it can take Entity A Key directly without using the simulation state
// It's body it's almost identical with the exception that it will first do a Passivate then it's normal execution
fn entity_b(shared_state: Rc<Cell<State>>, entity_a_key: Key, entity_states_key: StateKey<Passivated>) -> GenBoxed<()> {
    Box::new(move |_| {

        let mut state = shared_state.take();

        // Get a mutable borrow of the Passivated struct 
        let entity_states = state.get_mut(entity_states_key).unwrap();

        // Modify our state in the struct to indicate that it's doing a passivate
        entity_states.entity_b = true;

        // Return the extracted state back to the simulation
        // WARNING: This has to be done before a yield point or all other generators will recieve the default state without any value inside
        shared_state.set(state);

        // Emit the Passivate event
        println!("[ENTITY B] -> PASSIVATE");
        yield Action::Passivate;
        println!("[ENTITY B] <- PASSIVATE");

        // Same as above (excluding the yield)
        // but instead changing back our state to passivate = false
        let mut state = shared_state.take();
        state.get_mut(entity_states_key).unwrap().entity_b = false;
        shared_state.set(state);

        // Same as Entity A but with entity_a and entity_b swapped.
        loop {
            println!("[ENTITY B] -> HOLD");
            yield Action::Hold(Duration::from_secs(5));
            println!("[ENTITY B] <- HOLD");
            let mut state = shared_state.take();
            let entity_states = state.get_mut(entity_states_key).unwrap();
            if entity_states.entity_a {
                shared_state.set(state);
                println!("[ENTITY B] -> ACTIVATE [ENTITY A]");
                yield Action::ActivateOne(entity_a_key);
            } else {
                shared_state.set(state);
            }
            let mut state = shared_state.take();
            state.get_mut(entity_states_key).unwrap().entity_b = true;
            shared_state.set(state);
            println!("[ENTITY B] -> PASSIVATE");
            yield Action::Passivate;
            println!("[ENTITY B] <- PASSIVATE");
            let mut state = shared_state.take();
            state.get_mut(entity_states_key).unwrap().entity_b = false;
            shared_state.set(state);
        }
    })
}

// Helper struct to determine if entities are in passivate
pub struct Passivated {
    entity_a: bool,
    entity_b: bool,
}
