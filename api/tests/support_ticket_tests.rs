#![cfg(test)]
extern crate std;

use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    Address, Env, String, BytesN,
};
use api::{SupportTicketContract, SupportTicketContractClient, TicketCategory, TicketStatus};

// Helper to create a test environment with mock ledger time
fn create_test_env() -> Env {
    let env = Env::default();
    env.ledger().set(LedgerInfo {
        timestamp: 12345,
        protocol_version: 20,
        sequence_number: 10,
        network_id: BytesN::from_array(&env, &[0; 32]),
        base_reserve: 10,
    });
    env
}

#[test]
fn test_submit_ticket() {
    let env = create_test_env();
    let contract_id = env.register_contract(None, SupportTicketContract);
    let client = SupportTicketContractClient::new(&env, &contract_id);
    
    let user = Address::generate(&env);
    let category = TicketCategory::Payment;
    let subject = String::from_str(&env, "Payment Issue");
    let description = String::from_str(&env, "My payment was declined");
    
    // Submit a ticket
    let ticket = client.submit_ticket(&user, &category, &subject, &description);
    
    // Verify ticket details
    assert_eq!(ticket.id, 1);
    assert_eq!(ticket.user, user);
    assert_eq!(ticket.subject, subject);
    assert_eq!(ticket.description, description);
    assert_eq!(ticket.category, category);
    assert_eq!(ticket.status, TicketStatus::Open);
    assert_eq!(ticket.created_at, 12345);
    assert_eq!(ticket.admin, None);
    assert_eq!(ticket.closed_at, None);
}

#[test]
fn test_list_user_tickets() {
    let env = create_test_env();
    let contract_id = env.register_contract(None, SupportTicketContract);
    let client = SupportTicketContractClient::new(&env, &contract_id);
    
    let user = Address::generate(&env);
    
    // Submit two tickets
    client.submit_ticket(
        &user,
        &TicketCategory::Payment,
        &String::from_str(&env, "Payment Issue 1"),
        &String::from_str(&env, "Description 1"),
    );
    
    client.submit_ticket(
        &user,
        &TicketCategory::Course,
        &String::from_str(&env, "Course Issue"),
        &String::from_str(&env, "Description 2"),
    );
    
    // List user tickets
    let tickets = client.list_user_tickets(&user);
    
    // Verify ticket count
    assert_eq!(tickets.len(), 2);
    assert_eq!(tickets.get(0).subject, String::from_str(&env, "Payment Issue 1"));
    assert_eq!(tickets.get(1).subject, String::from_str(&env, "Course Issue"));
}

#[test]
fn test_admin_operations() {
    let env = create_test_env();
    let contract_id = env.register_contract(None, SupportTicketContract);
    let client = SupportTicketContractClient::new(&env, &contract_id);
    
    let user = Address::generate(&env);
    let admin = Address::generate(&env);
    
    // Submit a ticket
    client.submit_ticket(
        &user,
        &TicketCategory::Payment,
        &String::from_str(&env, "Payment Issue"),
        &String::from_str(&env, "Description"),
    );
    
    // Assign ticket to admin
    let assigned_ticket = client.assign_ticket(&admin, &1);
    assert_eq!(assigned_ticket.admin, Some(admin));
    assert_eq!(assigned_ticket.status, TicketStatus::InProgress);
    
    // Update ticket status to resolved
    let resolved_ticket = client.update_status(&admin, &1, &TicketStatus::Resolved);
    assert_eq!(resolved_ticket.status, TicketStatus::Resolved);
    
    // Close the ticket
    let closed_ticket = client.close_ticket(&admin, &1);
    assert_eq!(closed_ticket.status, TicketStatus::Closed);
    assert!(closed_ticket.closed_at.is_some());
    
    // Get open tickets (should be empty after closing)
    let open_tickets = client.list_open_tickets(&admin);
    assert_eq!(open_tickets.len(), 0);
}

#[test]
#[should_panic(expected = "Only resolved tickets can be closed")]
fn test_invalid_status_transition() {
    let env = create_test_env();
    let contract_id = env.register_contract(None, SupportTicketContract);
    let client = SupportTicketContractClient::new(&env, &contract_id);
    
    let user = Address::generate(&env);
    let admin = Address::generate(&env);
    
    // Submit a ticket
    client.submit_ticket(
        &user,
        &TicketCategory::Payment,
        &String::from_str(&env, "Payment Issue"),
        &String::from_str(&env, "Description"),
    );
    
    // Try to close a ticket that's not resolved (should panic)
    client.close_ticket(&admin, &1);
}

#[test]
fn test_delete_ticket() {
    let env = create_test_env();
    let contract_id = env.register_contract(None, SupportTicketContract);
    let client = SupportTicketContractClient::new(&env, &contract_id);
    
    let user = Address::generate(&env);
    let admin = Address::generate(&env);
    
    // Submit a ticket
    client.submit_ticket(
        &user,
        &TicketCategory::Payment,
        &String::from_str(&env, "Payment Issue"),
        &String::from_str(&env, "Description"),
    );
    
    // Delete the ticket
    client.delete_ticket(&admin, &1);
    
    // List user tickets (should be empty)
    let tickets = client.list_user_tickets(&user);
    assert_eq!(tickets.len(), 0);
}