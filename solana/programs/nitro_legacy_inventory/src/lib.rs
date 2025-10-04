use anchor_lang::prelude::*;

pub const CLASS_COUNT: usize = 4;
pub const SLOTS_PER_CLASS: usize = 20;

pub const MAX_ITEMS: usize = 64;
pub const MAX_NAME_LEN: usize = 32;
pub const MAX_ICON_LEN: usize = 8;
pub const MAX_DESCRIPTION_LEN: usize = 160;

// Replace with your deployed program id once available
declare_id!("EksA8EJMGvGQmbEWzQi9eXtviJZtPoEZk6VVVlsMC8UD");

#[program]
pub mod nitro_legacy_inventory {
    use super::*;

    pub fn initialize_registry(ctx: Context<InitializeRegistry>, registry_bump: u8) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        registry.authority = ctx.accounts.authority.key();
        registry.bump = registry_bump;
        registry.total_items = 0;
        registry.max_items = MAX_ITEMS as u16;
        registry.items = Vec::new();
        registry
            .slots
            .resize(InventoryRegistry::total_slots() as usize, SlotAssignment::default());
        Ok(())
    }

    pub fn create_item(ctx: Context<ModifyRegistry>, input: ItemInput) -> Result<u16> {
        let registry = &mut ctx.accounts.registry;
        require_keys_eq!(registry.authority, ctx.accounts.authority.key(), InventoryError::Unauthorized);
        require!(registry.items.len() < MAX_ITEMS, InventoryError::ItemCapacityReached);

        input.validate()?;

        let next_id = registry.total_items.checked_add(1).ok_or(InventoryError::ArithmeticOverflow)?;
        let item = Item::from_input(next_id, &input);
        registry.items.push(item);
        registry.total_items = next_id;
        Ok(next_id)
    }

    pub fn update_item(ctx: Context<ModifyRegistry>, item_id: u16, input: ItemInput, active: bool) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        require_keys_eq!(registry.authority, ctx.accounts.authority.key(), InventoryError::Unauthorized);
        input.validate()?;

        let item = registry
            .items
            .iter_mut()
            .find(|item| item.id == item_id)
            .ok_or(InventoryError::UnknownItem)?;

        item.update_from_input(&input, active);
        Ok(())
    }

    pub fn set_slot(ctx: Context<ModifyRegistry>, class_index: u8, slot_index: u8, item_id: Option<u16>) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        require_keys_eq!(registry.authority, ctx.accounts.authority.key(), InventoryError::Unauthorized);

        let slot_position = InventoryRegistry::slot_position(class_index, slot_index)? as usize;

        if let Some(target_id) = item_id {
            let item = registry
                .items
                .iter()
                .find(|item| item.id == target_id)
                .ok_or(InventoryError::UnknownItem)?;
            require!(item.active, InventoryError::InactiveItem);
        }

        let slot = &mut registry.slots[slot_position];
        match item_id {
            Some(id) => {
                slot.item_id = id;
                slot.occupied = true;
            }
            None => {
                *slot = SlotAssignment::default();
            }
        }

        Ok(())
    }

    pub fn clear_slot(ctx: Context<ModifyRegistry>, class_index: u8, slot_index: u8) -> Result<()> {
        set_slot(ctx, class_index, slot_index, None)
    }
}

#[derive(Accounts)]
pub struct InitializeRegistry<'info> {
    #[account(
        init,
        seeds = [b"nitro-registry", authority.key().as_ref()],
        bump,
        payer = authority,
        space = InventoryRegistry::space()
    )]
    pub registry: Account<'info, InventoryRegistry>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ModifyRegistry<'info> {
    #[account(mut, seeds = [b"nitro-registry", authority.key().as_ref()], bump = registry.bump)]
    pub registry: Account<'info, InventoryRegistry>,
    pub authority: Signer<'info>,
}

#[account]
pub struct InventoryRegistry {
    pub authority: Pubkey,
    pub bump: u8,
    pub total_items: u16,
    pub max_items: u16,
    pub items: Vec<Item>,
    pub slots: Vec<SlotAssignment>,
}

impl InventoryRegistry {
    pub fn total_slots() -> u16 {
        (CLASS_COUNT * SLOTS_PER_CLASS) as u16
    }

    pub const fn space() -> usize {
        // account discriminator + fields
        8 + // account discriminator
        32 + // authority
        1 + // bump
        2 + // total_items
        2 + // max_items
        4 + (MAX_ITEMS * Item::SIZE) + // Vec<Item>
        4 + (InventoryRegistry::total_slots() as usize * SlotAssignment::SIZE) // Vec<SlotAssignment>
    }

    pub fn slot_position(class_index: u8, slot_index: u8) -> Result<u16> {
        require!((class_index as usize) < CLASS_COUNT, InventoryError::InvalidClass);
        require!((slot_index as usize) < SLOTS_PER_CLASS, InventoryError::InvalidSlot);
        Ok((class_index as u16) * SLOTS_PER_CLASS as u16 + slot_index as u16)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct Item {
    pub id: u16,
    pub owner_code: u8,
    pub rarity: u8,
    pub active: bool,
    pub name: String,
    pub icon: String,
    pub description: String,
}

impl Item {
    pub const SIZE: usize = 2 + 1 + 1 + 1
        + 4 + MAX_NAME_LEN
        + 4 + MAX_ICON_LEN
        + 4 + MAX_DESCRIPTION_LEN;

    pub fn from_input(id: u16, input: &ItemInput) -> Self {
        Self {
            id,
            owner_code: input.owner_code,
            rarity: input.rarity,
            active: true,
            name: input.name.clone(),
            icon: input.icon.clone(),
            description: input.description.clone(),
        }
    }

    pub fn update_from_input(&mut self, input: &ItemInput, active: bool) {
        self.owner_code = input.owner_code;
        self.rarity = input.rarity;
        self.name = input.name.clone();
        self.icon = input.icon.clone();
        self.description = input.description.clone();
        self.active = active;
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ItemInput {
    pub owner_code: u8,
    pub rarity: u8,
    pub name: String,
    pub icon: String,
    pub description: String,
}

impl ItemInput {
    pub fn validate(&self) -> Result<()> {
        require!(self.owner_code <= CLASS_COUNT as u8, InventoryError::OwnerCodeOutOfRange);
        require!(self.name.len() <= MAX_NAME_LEN, InventoryError::NameTooLong);
        require!(self.icon.len() <= MAX_ICON_LEN, InventoryError::IconTooLong);
        require!(self.description.len() <= MAX_DESCRIPTION_LEN, InventoryError::DescriptionTooLong);
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct SlotAssignment {
    pub item_id: u16,
    pub occupied: bool,
}

impl SlotAssignment {
    pub const SIZE: usize = 2 + 1;
}

#[error_code]
pub enum InventoryError {
    #[msg("Unauthorized signer")]
    Unauthorized,
    #[msg("Item capacity reached")]
    ItemCapacityReached,
    #[msg("Item not found")]
    UnknownItem,
    #[msg("Inactive item cannot be equipped")]
    InactiveItem,
    #[msg("Owner code exceeds allowed range")]
    OwnerCodeOutOfRange,
    #[msg("Class index is invalid")]
    InvalidClass,
    #[msg("Slot index is invalid")]
    InvalidSlot,
    #[msg("Item name too long")]
    NameTooLong,
    #[msg("Item icon too long")]
    IconTooLong,
    #[msg("Item description too long")]
    DescriptionTooLong,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
}
