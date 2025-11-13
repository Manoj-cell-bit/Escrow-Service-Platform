#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Address, Symbol, symbol_short};

// Escrow structure to track escrow details
#[contracttype]
#[derive(Clone)]
pub struct Escrow {
    pub escrow_id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub amount: i128,
    pub is_active: bool,
    pub is_released: bool,
    pub is_refunded: bool,
    pub created_at: u64,
}

// Mapping escrow_id to Escrow details
#[contracttype]
pub enum EscrowBook {
    Escrow(u64)
}

// Counter for generating unique escrow IDs
const ESCROW_COUNT: Symbol = symbol_short!("ESC_CNT");

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    
    /// Creates a new escrow with buyer, seller, and amount
    /// Returns the unique escrow_id
    pub fn create_escrow(env: Env, buyer: Address, seller: Address, amount: i128) -> u64 {
        // Verify the buyer is calling this function
        buyer.require_auth();
        
        // Get and increment escrow counter
        let mut escrow_count: u64 = env.storage().instance().get(&ESCROW_COUNT).unwrap_or(0);
        escrow_count += 1;
        
        // Get current timestamp
        let timestamp = env.ledger().timestamp();
        
        // Create new escrow instance
        let new_escrow = Escrow {
            escrow_id: escrow_count,
            buyer: buyer.clone(),
            seller: seller.clone(),
            amount,
            is_active: true,
            is_released: false,
            is_refunded: false,
            created_at: timestamp,
        };
        
        // Store the escrow
        env.storage().instance().set(&EscrowBook::Escrow(escrow_count), &new_escrow);
        env.storage().instance().set(&ESCROW_COUNT, &escrow_count);
        
        // Extend storage TTL
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Escrow Created with ID: {}", escrow_count);
        escrow_count
    }
    
    /// Release funds from escrow to seller
    /// Can only be called by the buyer
    pub fn release_funds(env: Env, escrow_id: u64) {
        let mut escrow = Self::view_escrow(env.clone(), escrow_id);
        
        // Verify the buyer is calling this function
        escrow.buyer.require_auth();
        
        // Check if escrow is active and not already released or refunded
        if escrow.is_active && !escrow.is_released && !escrow.is_refunded {
            escrow.is_released = true;
            escrow.is_active = false;
            
            // Update escrow status
            env.storage().instance().set(&EscrowBook::Escrow(escrow_id), &escrow);
            env.storage().instance().extend_ttl(5000, 5000);
            
            log!(&env, "Escrow ID: {} - Funds released to seller", escrow_id);
        } else {
            log!(&env, "Cannot release funds - Invalid escrow state");
            panic!("Cannot release funds - Invalid escrow state");
        }
    }
    
    /// Refund funds from escrow to buyer
    /// Can be called by either buyer or seller
    pub fn refund_escrow(env: Env, escrow_id: u64, caller: Address) {
        let mut escrow = Self::view_escrow(env.clone(), escrow_id);
        
        // Verify the caller is either buyer or seller
        caller.require_auth();
        
        // Check if caller is authorized (buyer or seller)
        if caller != escrow.buyer && caller != escrow.seller {
            log!(&env, "Unauthorized caller");
            panic!("Unauthorized: Only buyer or seller can refund");
        }
        
        // Check if escrow is active and not already released or refunded
        if escrow.is_active && !escrow.is_released && !escrow.is_refunded {
            escrow.is_refunded = true;
            escrow.is_active = false;
            
            // Update escrow status
            env.storage().instance().set(&EscrowBook::Escrow(escrow_id), &escrow);
            env.storage().instance().extend_ttl(5000, 5000);
            
            log!(&env, "Escrow ID: {} - Funds refunded to buyer", escrow_id);
        } else {
            log!(&env, "Cannot refund - Invalid escrow state");
            panic!("Cannot refund - Invalid escrow state");
        }
    }
    
    /// View escrow details by escrow_id
    pub fn view_escrow(env: Env, escrow_id: u64) -> Escrow {
        let key = EscrowBook::Escrow(escrow_id);
        
        env.storage().instance().get(&key).unwrap_or_else(|| {
            log!(&env, "Escrow not found");
            panic!("Escrow not found with ID: {}", escrow_id);
        })
    }
}
