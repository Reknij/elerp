# Quick Start
## Interfaces
Click the menu button in the upper left corner to open the menu.
![Click Menu](../assets/click-menu.png)

And click to enter the specified interface.
![Interfaces](../assets/interfaces.png)

## Dependecies
Before adding the first order, let's add the following, at least one each.
- Area
- Person
- Warehouse
- Order Category
- SKU Category
- SKU

Each dependency above will have two columns:
- `Color` is optional. It is background color of dependency. Default is gray.
- `Text Color` is optional. It is text color of dependency. Default is black.

## Add Area
- `Name` is required. It is the area's name.
- `Description` is optional.

## Add Person
- `Name` is required. It is the area's name.
- `Description` is optional.
- `Person in charge` is optional. It is person in charge for this person.
- `Area` is required. It is area of person.
- `Address` is optional. It is address of person.
- `Contact number` is optional. It is contact number of person.
- `Email` is optional. It is email of person.

## Add Warehouse
- `Area` is required. It is area of warehouse.
- `Person in charge` is required. It is person in charge for this warehouse.
- `Name` is required. It is the warehouse's name.
- `Description` is optional.
- `Address` is optional. It is address of warehouse.

## Add SKU Category
- `Name` is required. It is the SKU category's name.
- `Description` is optional.

## Add SKU
- `SKU Category` is required. It is category of SKU.
- `Name` is required. It is the SKU's name.
- `Description` is optional.

## Add Order Category
- `Name` is required. It is the order's name.
- `Description` is optional.