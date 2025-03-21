#![no_std]
use soroban_sdk::{contractimpl, contracttype, Address, BytesN, Env, String, Symbol, Vec, Map};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Ticket(u64),
    UserTickets(Address),
    OpenTickets,
    TicketCounter,
}

#[derive(Clone)]
#[contracttype]
pub enum TicketCategory {
    Payment,
    Course,
    Certificate,
    Account,
    Other,
}

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum TicketStatus {
    Open,
    InProgress,
    Resolved,
    Closed,
}

#[derive(Clone)]
#[contracttype]
pub struct SupportTicket {
    pub id: u64,
    pub user: Address,
    pub admin: Option<Address>,
    pub subject: String,
    pub description: String,
    pub category: TicketCategory,
    pub status: TicketStatus,
    pub created_at: u64, // Timestamp
    pub updated_at: u64, // Timestamp
    pub closed_at: Option<u64>, // Timestamp
}

pub struct SupportTicketContract;

#[contractimpl]
impl SupportTicketContract {
    // Submit a support ticket
    pub fn submit_ticket(
        env: Env,
        user: Address,
        category: TicketCategory,
        subject: String,
        description: String,
    ) -> SupportTicket {
        // Verify caller is the user
        user.require_auth();
        
        // Get current timestamp
        let now = env.ledger().timestamp();
        
        // Get next ticket ID
        let ticket_id = Self::get_next_id(&env);
        
        // Create new ticket
        let ticket = SupportTicket {
            id: ticket_id,
            user: user.clone(),
            admin: None,
            subject,
            description,
            category,
            status: TicketStatus::Open,
            created_at: now,
            updated_at: now,
            closed_at: None,
        };
        
        // Store ticket
        env.storage().set(&DataKey::Ticket(ticket_id), &ticket);
        
        // Add to user's tickets
        let mut user_tickets = Self::get_user_tickets(&env, &user);
        user_tickets.push_back(ticket_id);
        env.storage().set(&DataKey::UserTickets(user), &user_tickets);
        
        // Add to open tickets
        let mut open_tickets = Self::get_open_tickets(&env);
        open_tickets.push_back(ticket_id);
        env.storage().set(&DataKey::OpenTickets, &open_tickets);
        
        ticket
    }
    
    // Get ticket by ID
    pub fn get_ticket(env: Env, ticket_id: u64) -> SupportTicket {
        env.storage().get(&DataKey::Ticket(ticket_id))
            .unwrap_or_else(|| panic!("Ticket not found"))
    }
    
    // List all tickets for a user
    pub fn list_user_tickets(env: Env, user: Address) -> Vec<SupportTicket> {
        let ticket_ids = Self::get_user_tickets(&env, &user);
        let mut tickets = Vec::new(&env);
        
        for id in ticket_ids.iter() {
            let ticket: SupportTicket = env.storage().get(&DataKey::Ticket(id)).unwrap();
            tickets.push_back(ticket);
        }
        
        tickets
    }
    
    // List open tickets (admin only)
    pub fn list_open_tickets(env: Env, admin: Address) -> Vec<SupportTicket> {
        // Verify caller is an admin
        Self::require_admin(&env, &admin);
        
        let ticket_ids = Self::get_open_tickets(&env);
        let mut tickets = Vec::new(&env);
        
        for id in ticket_ids.iter() {
            let ticket: SupportTicket = env.storage().get(&DataKey::Ticket(id)).unwrap();
            tickets.push_back(ticket);
        }
        
        tickets
    }
    
    // Assign ticket to admin
    pub fn assign_ticket(env: Env, admin: Address, ticket_id: u64) -> SupportTicket {
        // Verify caller is an admin
        Self::require_admin(&env, &admin);
        
        // Get ticket
        let mut ticket: SupportTicket = env.storage().get(&DataKey::Ticket(ticket_id))
            .unwrap_or_else(|| panic!("Ticket not found"));
        
        // Update ticket
        ticket.admin = Some(admin);
        
        // If the ticket is open, move to in progress
        if ticket.status == TicketStatus::Open {
            ticket.status = TicketStatus::InProgress;
            
            // Update open tickets list if status changed
            let mut open_tickets = Self::get_open_tickets(&env);
            // Keep ticket in open tickets list as "InProgress" is still considered "open"
        }
        
        ticket.updated_at = env.ledger().timestamp();
        
        // Save updated ticket
        env.storage().set(&DataKey::Ticket(ticket_id), &ticket);
        
        ticket
    }
    
    // Update ticket status
    pub fn update_status(env: Env, admin: Address, ticket_id: u64, status: TicketStatus) -> SupportTicket {
        // Verify caller is an admin
        Self::require_admin(&env, &admin);
        
        // Get ticket
        let mut ticket: SupportTicket = env.storage().get(&DataKey::Ticket(ticket_id))
            .unwrap_or_else(|| panic!("Ticket not found"));
        
        // Check valid status transitions
        match (&ticket.status, &status) {
            // Valid transitions
            (TicketStatus::Open, TicketStatus::InProgress) => {},
            (TicketStatus::Open, TicketStatus::Resolved) => {},
            (TicketStatus::InProgress, TicketStatus::Resolved) => {},
            (TicketStatus::Resolved, TicketStatus::Closed) => {
                ticket.closed_at = Some(env.ledger().timestamp());
                
                // Remove from open tickets if closed
                let mut open_tickets = Self::get_open_tickets(&env);
                let mut new_open_tickets = Vec::new(&env);
                
                for id in open_tickets.iter() {
                    if id != ticket_id {
                        new_open_tickets.push_back(id);
                    }
                }
                
                env.storage().set(&DataKey::OpenTickets, &new_open_tickets);
            },
            (TicketStatus::InProgress, TicketStatus::Open) => {},
            (TicketStatus::Resolved, TicketStatus::InProgress) => {},
            // Invalid transitions
            _ => panic!("Invalid status transition"),
        }
        
        // Update ticket
        ticket.status = status;
        ticket.updated_at = env.ledger().timestamp();
        
        // Save updated ticket
        env.storage().set(&DataKey::Ticket(ticket_id), &ticket);
        
        ticket
    }
    
    // Close ticket
    pub fn close_ticket(env: Env, admin: Address, ticket_id: u64) -> SupportTicket {
        // Verify caller is an admin
        Self::require_admin(&env, &admin);
        
        // Get ticket
        let mut ticket: SupportTicket = env.storage().get(&DataKey::Ticket(ticket_id))
            .unwrap_or_else(|| panic!("Ticket not found"));
        
        // Only resolved tickets can be closed
        if ticket.status != TicketStatus::Resolved {
            panic!("Only resolved tickets can be closed");
        }
        
        // Update ticket
        ticket.status = TicketStatus::Closed;
        let now = env.ledger().timestamp();
        ticket.closed_at = Some(now);
        ticket.updated_at = now;
        
        // Save updated ticket
        env.storage().set(&DataKey::Ticket(ticket_id), &ticket);
        
        // Remove from open tickets
        let mut open_tickets = Self::get_open_tickets(&env);
        let mut new_open_tickets = Vec::new(&env);
        
        for id in open_tickets.iter() {
            if id != ticket_id {
                new_open_tickets.push_back(id);
            }
        }
        
        env.storage().set(&DataKey::OpenTickets, &new_open_tickets);
        
        ticket
    }
    
    // Delete ticket (admin only)
    pub fn delete_ticket(env: Env, admin: Address, ticket_id: u64) {
        // Verify caller is an admin
        Self::require_admin(&env, &admin);
        
        // Get ticket to verify it exists
        let ticket: SupportTicket = env.storage().get(&DataKey::Ticket(ticket_id))
            .unwrap_or_else(|| panic!("Ticket not found"));
        
        // Remove from user's tickets
        let mut user_tickets = Self::get_user_tickets(&env, &ticket.user);
        let mut new_user_tickets = Vec::new(&env);
        
        for id in user_tickets.iter() {
            if id != ticket_id {
                new_user_tickets.push_back(id);
            }
        }
        
        env.storage().set(&DataKey::UserTickets(ticket.user), &new_user_tickets);
        
        // Remove from open tickets if not closed
        if ticket.status != TicketStatus::Closed {
            let mut open_tickets = Self::get_open_tickets(&env);
            let mut new_open_tickets = Vec::new(&env);
            
            for id in open_tickets.iter() {
                if id != ticket_id {
                    new_open_tickets.push_back(id);
                }
            }
            
            env.storage().set(&DataKey::OpenTickets, &new_open_tickets);
        }
        
        // Delete ticket
        env.storage().remove(&DataKey::Ticket(ticket_id));
    }
    
    // Private helper functions
    
    fn get_next_id(env: &Env) -> u64 {
        let counter: u64 = env.storage().get(&DataKey::TicketCounter).unwrap_or(0);
        let new_counter = counter + 1;
        env.storage().set(&DataKey::TicketCounter, &new_counter);
        new_counter
    }
    
    fn get_user_tickets(env: &Env, user: &Address) -> Vec<u64> {
        env.storage().get(&DataKey::UserTickets(user.clone())).unwrap_or_else(|| Vec::new(env))
    }
    
    fn get_open_tickets(env: &Env) -> Vec<u64> {
        env.storage().get(&DataKey::OpenTickets).unwrap_or_else(|| Vec::new(env))
    }
    
    fn require_admin(env: &Env, admin: &Address) {
        // In a real implementation, you would check if the address is an admin
        // For simplicity, we just check authentication
        admin.require_auth();
        
        // TODO: Add proper admin role checking
        // Example:
        // let is_admin: bool = env.storage().get(&DataKey::Admin(admin.clone())).unwrap_or(false);
        // if !is_admin {
        //     panic!("Address is not an admin");
        // }
    }
}