#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Project {
    id: u64,
    name: String,
    description: String,
    credits: u64,
    price: u64,
    owner: String,
    verified: bool,
}

impl Storable for Project {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Project {
    const MAX_SIZE: u32 = 2048; // Adjust based on expected data size
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Credit {
    id: u64,
    project_id: u64,
    amount: u64,
    price: u64,
    issued_to: String,
    owner: String,
    is_for_sale: bool,
}

impl Storable for Credit {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Credit {
    const MAX_SIZE: u32 = 2048; // Adjust based on expected data size
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct CreditForSale {
    id: u64,
    credit_id: u64,
    amount: u64,
    price: u64,
    seller: String,
}

impl Storable for CreditForSale {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for CreditForSale {
    const MAX_SIZE: u32 = 2048; // Adjust based on expected data size
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct User {
    id: u64,
    name: String,
    credit_balance: u64,
}

impl Storable for User {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for User {
    const MAX_SIZE: u32 = 2048; // Adjust based on expected data size
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static PROJECTS: RefCell<StableBTreeMap<u64, Project, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))))
    );

    static CREDITS: RefCell<StableBTreeMap<u64, Credit, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))))
    );

    static CREDITS_FOR_SALE: RefCell<StableBTreeMap<u64, CreditForSale, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))))
    );

    static USERS: RefCell<StableBTreeMap<u64, User, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5))))
    );
}

#[ic_cdk::update]
fn register_project(name: String, description: String, credits: u64, price: u64, owner: String) -> Option<Project> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");

    let project = Project {
        id,
        name,
        description,
        credits,
        price,
        owner,
        verified: false,
    };

    PROJECTS.with(|projects| {
        projects.borrow_mut().insert(id, project.clone());
    });

    Some(project)
}

#[ic_cdk::query]
fn read_project(project_id: u64) -> Result<Project, String> {
    if let Some(project) = PROJECTS.with(|projects| projects.borrow().get(&project_id)) {
        Ok(project.clone())
    } else {
        Err(format!("Project with id={} not found", project_id))
    }
}

#[ic_cdk::update]
fn update_project(project_id: u64, name: String, description: String, credits: u64, price: u64, owner: String, verified: bool) -> Result<Project, String> {
    PROJECTS.with(|projects| {
        let mut projects = projects.borrow_mut();

        if let Some(mut project) = projects.remove(&project_id) {
            project.name = name;
            project.description = description;
            project.credits = credits;
            project.price = price;
            project.owner = owner;
            project.verified = verified;

            projects.insert(project_id, project.clone());

            Ok(project)
        } else {
            Err(format!("Project with id={} not found", project_id))
        }
    })
}

#[ic_cdk::update]
fn delete_project(project_id: u64) -> Result<Project, String> {
    PROJECTS
        .with(|projects| projects.borrow_mut().remove(&project_id))
        .ok_or(format!("Project with id={} not found", project_id))
}

#[ic_cdk::update]
fn issue_credit(project_id: u64, amount: u64, price: u64, issued_to: String) -> Option<Credit> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");

    let credit = Credit {
        id,
        project_id,
        amount,
        price,
        owner: issued_to.clone(), // Clone issued_to before it's moved
        issued_to: issued_to.clone(),
        is_for_sale: false,
    };

    CREDITS.with(|credits| {
        credits.borrow_mut().insert(id, credit.clone());
    });

    Some(credit)
}

#[ic_cdk::query]
fn read_credit(credit_id: u64) -> Result<Credit, String> {
    if let Some(credit) = CREDITS.with(|credits| credits.borrow().get(&credit_id)) {
        Ok(credit.clone())
    } else {
        Err(format!("Credit with id={} not found", credit_id))
    }
}

#[ic_cdk::update]
fn update_credit(credit_id: u64, project_id: u64, amount: u64, price: u64, issued_to: String, owner: String, is_for_sale: bool) -> Result<Credit, String> {
    CREDITS.with(|credits| {
        let mut credits = credits.borrow_mut();

        if let Some(mut credit) = credits.remove(&credit_id) {
            credit.project_id = project_id;
            credit.amount = amount;
            credit.price = price;
            credit.issued_to = issued_to;
            credit.owner = owner;
            credit.is_for_sale = is_for_sale;

            credits.insert(credit_id, credit.clone());

            Ok(credit)
        } else {
            Err(format!("Credit with id={} not found", credit_id))
        }
    })
}

#[ic_cdk::update]
fn delete_credit(credit_id: u64) -> Result<Credit, String> {
    CREDITS
        .with(|credits| credits.borrow_mut().remove(&credit_id))
        .ok_or(format!("Credit with id={} not found", credit_id))
}

#[ic_cdk::update]
fn list_credit_for_sale(credit_id: u64, amount: u64, price: u64, seller: String) -> Option<CreditForSale> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");

    let credit_for_sale = CreditForSale {
        id,
        credit_id,
        amount,
        price,
        seller,
    };

    CREDITS_FOR_SALE.with(|credits| {
        credits.borrow_mut().insert(id, credit_for_sale.clone());
    });

    Some(credit_for_sale)
}

#[ic_cdk::query]
fn read_credit_for_sale(credit_for_sale_id: u64) -> Result<CreditForSale, String> {
    if let Some(credit_for_sale) = CREDITS_FOR_SALE.with(|credits| credits.borrow().get(&credit_for_sale_id)) {
        Ok(credit_for_sale.clone())
    } else {
        Err(format!("CreditForSale with id={} not found", credit_for_sale_id))
    }
}

#[ic_cdk::update]
fn update_credit_for_sale(credit_for_sale_id: u64, credit_id: u64, amount: u64, price: u64, seller: String) -> Result<CreditForSale, String> {
    CREDITS_FOR_SALE.with(|credits| {
        let mut credits = credits.borrow_mut();

        if let Some(mut credit_for_sale) = credits.remove(&credit_for_sale_id) {
            credit_for_sale.credit_id = credit_id;
            credit_for_sale.amount = amount;
            credit_for_sale.price = price;
            credit_for_sale.seller = seller;

            credits.insert(credit_for_sale_id, credit_for_sale.clone());

            Ok(credit_for_sale)
        } else {
            Err(format!("CreditForSale with id={} not found", credit_for_sale_id))
        }
    })
}

#[ic_cdk::update]
fn delete_credit_for_sale(credit_for_sale_id: u64) -> Result<CreditForSale, String> {
    CREDITS_FOR_SALE
        .with(|credits| credits.borrow_mut().remove(&credit_for_sale_id))
        .ok_or(format!("CreditForSale with id={} not found", credit_for_sale_id))
}

#[ic_cdk::update]
fn buy_credits(credit_for_sale_id: u64, buyer_id: u64) -> Result<(), String> {
    let credit_for_sale = CREDITS_FOR_SALE.with(|credits| credits.borrow_mut().remove(&credit_for_sale_id));
    let buyer = USERS.with(|users| users.borrow_mut().remove(&buyer_id));

    match (credit_for_sale, buyer) {
        (Some(credit_for_sale), Some(mut buyer)) => {
            if credit_for_sale.price <= buyer.credit_balance {
                // Deduct price from buyer's balance
                buyer.credit_balance -= credit_for_sale.price;

                // Transfer ownership
                let mut credit = CREDITS.with(|credits| credits.borrow_mut().remove(&credit_for_sale.credit_id)).ok_or("Credit not found")?;
                credit.owner = buyer.name.clone();

                // Update records
                USERS.with(|users| {
                    users.borrow_mut().insert(buyer_id, buyer);
                });
                CREDITS.with(|credits| {
                    credits.borrow_mut().insert(credit_for_sale.credit_id, credit);
                });

                Ok(())
            } else {
                // Reinsert the credit_for_sale back as the transaction failed
                CREDITS_FOR_SALE.with(|credits| {
                    credits.borrow_mut().insert(credit_for_sale_id, credit_for_sale);
                });

                Err("Insufficient balance".to_string())
            }
        }
        _ => Err("Credit or buyer not found".to_string()),
    }
}



#[ic_cdk::query]
fn track_credits() -> Vec<Credit> {
    CREDITS.with(|credits| credits.borrow().iter().map(|(_, credit)| credit.clone()).collect())
}

#[ic_cdk::update]
fn verify_project(project_id: u64) -> Result<Project, String> {
    PROJECTS.with(|projects| {
        let mut projects = projects.borrow_mut();

        if let Some(mut project) = projects.remove(&project_id) {
            project.verified = true;

            projects.insert(project_id, project.clone());

            Ok(project)
        } else {
            Err(format!("Project with id={} not found", project_id))
        }
    })
}

#[ic_cdk::query]
fn query_credit(credit_id: u64) -> Result<Credit, String> {
    if let Some(credit) = CREDITS.with(|credits| credits.borrow().get(&credit_id)) {
        Ok(credit.clone())
    } else {
        Err(format!("Credit with id={} not found", credit_id))
    }
}

#[ic_cdk::query]
fn calculate_carbon_footprint(user_id: u64) -> Result<u64, String> {
    let user_option = USERS.with(|users| users.borrow_mut().remove(&user_id));

    if let Some(user) = user_option {
        // Calculate the carbon footprint based on user activities or other criteria
        // Placeholder logic: total credits owned by the user
        let total_credits: u64 = CREDITS.with(|credits| {
            credits.borrow().iter()
                .filter(|(_, credit)| credit.owner == user.name)
                .map(|(_, credit)| credit.amount)
                .sum()
        });

        // Reinsert the user back after calculation
        USERS.with(|users| {
            users.borrow_mut().insert(user_id, user);
        });

        Ok(total_credits)
    } else {
        Err(format!("User with id={} not found", user_id))
    }
}

ic_cdk::export_candid!();
