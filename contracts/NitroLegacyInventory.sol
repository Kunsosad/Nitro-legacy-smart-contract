// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @title Nitro Legacy Inventory Registry
/// @notice Stores game inventory metadata and slot assignments for the Nitro Legacy front-end.
contract NitroLegacyInventory {
    enum ClassId {
        Swordfighter,
        Archer,
        Magician,
        Support
    }

    enum Rarity {
        Common,
        Rare,
        Epic,
        Legendary
    }

    struct Item {
        uint256 id;
        string name;
        string icon;
        string description;
        Rarity rarity;
        uint8 ownerCode; // 0 = all classes, otherwise 1-4 match ClassId order.
        bool active;
    }

    struct InventorySlot {
        uint8 slot;
        uint256 itemId;
        bool hasItem;
    }

    address private _owner;
    uint256 private _nextItemId = 1;

    mapping(uint256 => Item) private _items;
    uint256[] private _itemIds;

    mapping(ClassId => mapping(uint8 => uint256)) private _slotAssignments;
    mapping(ClassId => mapping(uint8 => bool)) private _slotRegistered;
    mapping(ClassId => uint8[]) private _classSlots;

    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    event ItemCreated(uint256 indexed itemId, string name, uint8 ownerCode, Rarity rarity);
    event ItemUpdated(uint256 indexed itemId, string name, uint8 ownerCode, Rarity rarity, bool active);
    event SlotSet(ClassId indexed classId, uint8 indexed slot, uint256 itemId);
    event SlotCleared(ClassId indexed classId, uint8 indexed slot);

    modifier onlyOwner() {
        require(msg.sender == _owner, "Not authorized");
        _;
    }

    constructor() {
        _owner = msg.sender;
        emit OwnershipTransferred(address(0), msg.sender);
    }

    /*//////////////////////////////////////////////////////////////
                                OWNERSHIP
    //////////////////////////////////////////////////////////////*/

    function owner() external view returns (address) {
        return _owner;
    }

    function transferOwnership(address newOwner) external onlyOwner {
        require(newOwner != address(0), "Invalid owner");
        emit OwnershipTransferred(_owner, newOwner);
        _owner = newOwner;
    }

    /*//////////////////////////////////////////////////////////////
                                 ITEMS
    //////////////////////////////////////////////////////////////*/

    function createItem(
        string calldata name_,
        string calldata icon_,
        string calldata description_,
        Rarity rarity_,
        uint8 ownerCode_
    ) external onlyOwner returns (uint256 itemId) {
        require(bytes(name_).length > 0, "Name required");
        require(ownerCode_ <= 4, "Owner code out of range");

        itemId = _nextItemId++;

        Item storage item = _items[itemId];
        item.id = itemId;
        item.name = name_;
        item.icon = icon_;
        item.description = description_;
        item.rarity = rarity_;
        item.ownerCode = ownerCode_;
        item.active = true;

        _itemIds.push(itemId);

        emit ItemCreated(itemId, name_, ownerCode_, rarity_);
    }

    function updateItem(
        uint256 itemId,
        string calldata name_,
        string calldata icon_,
        string calldata description_,
        Rarity rarity_,
        uint8 ownerCode_,
        bool active_
    ) external onlyOwner {
        Item storage item = _items[itemId];
        require(item.id != 0, "Unknown item");
        require(bytes(name_).length > 0, "Name required");
        require(ownerCode_ <= 4, "Owner code out of range");

        item.name = name_;
        item.icon = icon_;
        item.description = description_;
        item.rarity = rarity_;
        item.ownerCode = ownerCode_;
        item.active = active_;

        emit ItemUpdated(itemId, name_, ownerCode_, rarity_, active_);
    }

    function getItem(uint256 itemId) external view returns (Item memory) {
        Item memory item = _items[itemId];
        require(item.id != 0, "Unknown item");
        return item;
    }

    function listItems() external view returns (Item[] memory items) {
        uint256 total = _itemIds.length;
        items = new Item[](total);
        for (uint256 i = 0; i < total; i++) {
            items[i] = _items[_itemIds[i]];
        }
    }

    /*//////////////////////////////////////////////////////////////
                                INVENTORY
    //////////////////////////////////////////////////////////////*/

    function setInventorySlot(ClassId classId, uint8 slot, uint256 itemId) external onlyOwner {
        require(slot > 0, "Slot must be > 0");
        if (itemId != 0) {
            Item memory item = _items[itemId];
            require(item.id != 0, "Unknown item");
            require(item.active, "Item inactive");
        }

        if (!_slotRegistered[classId][slot]) {
            _slotRegistered[classId][slot] = true;
            _classSlots[classId].push(slot);
        }

        _slotAssignments[classId][slot] = itemId;

        if (itemId == 0) {
            emit SlotCleared(classId, slot);
        } else {
            emit SlotSet(classId, slot, itemId);
        }
    }

    function clearInventorySlot(ClassId classId, uint8 slot) external onlyOwner {
        require(_slotRegistered[classId][slot], "Slot unknown");
        _slotAssignments[classId][slot] = 0;
        emit SlotCleared(classId, slot);
    }

    function getInventory(ClassId classId) external view returns (InventorySlot[] memory slots) {
        uint256 total = _classSlots[classId].length;
        slots = new InventorySlot[](total);
        for (uint256 i = 0; i < total; i++) {
            uint8 slotNumber = _classSlots[classId][i];
            uint256 itemId = _slotAssignments[classId][slotNumber];
            slots[i] = InventorySlot({
                slot: slotNumber,
                itemId: itemId,
                hasItem: itemId != 0
            });
        }
    }

    function getClassSlots(ClassId classId) external view returns (uint8[] memory) {
        return _classSlots[classId];
    }

    function slotItemId(ClassId classId, uint8 slot) external view returns (uint256) {
        require(_slotRegistered[classId][slot], "Slot unknown");
        return _slotAssignments[classId][slot];
    }
}
