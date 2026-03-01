// Phase 60c: Verify MainController stores components directly (no more Box::leak).
//
// Previously these methods used Box::leak to return references.
// Now they return references to owned fields, so repeated calls return the same address.

use beatoraja_result::stubs::{MainController, NullMainController};

/// get_input_processor() returns the same stored instance on repeated calls.
#[test]
fn get_input_processor_returns_same_instance() {
    let mut mc = MainController::new(Box::new(NullMainController));

    let ptr1 = mc.get_input_processor() as *const _ as usize;
    let ptr2 = mc.get_input_processor() as *const _ as usize;

    assert_eq!(ptr1, ptr2, "should return same stored instance");
}

/// ir_send_status() returns the same stored Vec on repeated calls.
#[test]
fn ir_send_status_returns_same_instance() {
    let mc = MainController::new(Box::new(NullMainController));

    let ptr1 = mc.ir_send_status() as *const _ as usize;
    let ptr2 = mc.ir_send_status() as *const _ as usize;

    assert_eq!(ptr1, ptr2, "should return same stored Vec");
}

/// get_play_data_accessor() returns the same stored instance on repeated calls.
#[test]
fn get_play_data_accessor_returns_same_instance() {
    let mc = MainController::new(Box::new(NullMainController));

    let ptr1 = mc.get_play_data_accessor() as *const _ as usize;
    let ptr2 = mc.get_play_data_accessor() as *const _ as usize;

    assert_eq!(ptr1, ptr2, "should return same stored instance");
}
