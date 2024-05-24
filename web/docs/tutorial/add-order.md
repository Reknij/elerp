# Let Add The First Order
## Linking your user to the warehouse
If you have a general user and want to give them permission to add orders, you may have enabled the "Add Order" permission for that user, only to find that the user cannot find any warehouses.  This is because every warehouse is hidden by default and you must link the user to the specified warehouse before the user can access the warehouse.

![Linking user to warehouse](../assets/linking-user-to-warehouse.png)

## Order Information
The columns of an order are as follows:
- `Order Type` is what type of order.
    - `Stock In` is stock in order. Used when you want stock in the products.
    - `Stock Out` is stock out order. Mostly used.
    - `Return` is return order. Used when you want return some products. It does the same thing as `Stock In` but with the flag `Return`.
    - `Exchange` is exchange order. Used when you want exchange some products. For example, your customer returns `Cola` to you and exchanges it for `Sarsi`.  So your inventory will be +1 for `Cola` and -1 for `Sarsi`.
    - `Calibration` is calibrating your inventory. Please use it with caution. Usually because your inventory is added or reduced for special reasons and does not belong to `Stock In`/`Stock Out`, you can use `Calibration` to directly change the specified product inventory to what you want.
    - `Calibration Strict` is calibrating your inventory. It does the same thing as `Calibration` but it is strict! Change the specified product inventory to what you want, and other products will change to **zero**!
    - `Verification` is used to verify your inventory. Safe to use, nothing will happen.
    - `Verification Strict` is used to verify your inventory. But it is strict! Unspecified product inventory must **zero**. Safe to use, nothing will happen.
- `Order Category` is what category of order. For example, you can add `General` category for all general order.
- `Warehouse` is which warehouse is processing this order. It will calculate the inventory of the target warehouse.
- `Person Related` is which person related to this order. If you are a reseller, you can add your customer information before this. So this column is who placed the order. Otherwise, you can just add a person for "All Customers" and select this person
- `Description` is optional. Write your description for this order.
- `Currency` is what currency used in this order.
- `Items` is the order's goods. Add your order's products here. Empty is allowed.
    - Select the product's SKU.
    - Write the product's price.
    - Write the product's quantity.

::: tip
You want add multiple once time? You can click `Add multiple item` button. In the modal, select the SKU category you want add, write the items with specified format:
```
Ice cola 4
Ice sarsi 2
```
Write your price, for example `5`, It will add 4 ice cola, 2 ice sarsi with price 5.

**Since some SKUs may have the same name but different categories, you need to select the SKU category.**
:::

::: warning
All orders can be checked to see if the inventory is satisfied before adding, and will be checked again in the system after clicking add. But manual check can quickly provide sufficient information about which products are not satisfied.
:::