# Admin And General User

## Admin
`Admin` is default user for `Elerp`. Please remember the username and password and don't forget it.

**The system only can exists one `Admin`.**

## General User
All general user can be created by `Admin`. Click `Add` button, fill the form then click the `Add` button to confirm add.

Form columns:
- `Alias` is alias of user.
- `Username` is username of user. Must be unique.
- `Password` is password of user. At least 8 characters and at least 1 number and letter.

## Permission of general user
Check the specified checkbox to enable the relevant permissions:
- `Manage Area` checked means user can add/remove/update areas.
- `Manage Person` checked means user can add/remove/update persons.
::: warning
When `Manage Person` is enabled, users can see all information about the person.  Otherwise the address and contact details are not visible.
:::
- `Manage Warehouse` checked means user can add/remove/update warehouses.
::: warning
When `Manage Warehouse` is enabled, users can see all information about the warehouse.  Otherwise the address details are not visible. Enabling `Manage Repository` means that the user has permissions for the all warehouses, it's like linking all warehouses to the user.
:::
- `Manage SKU` checked means user can add/remove/update SKUs.
- `Manage SKU Category` checked means user can add/remove/update SKU categories.
- `Manage Order Category` checked means user can add/remove/update order categories.
- `Add Order` checked means user can add order.
- `Update And Remove Order` checked means user can update/remove order.
- `Add Order Payment` checked means user can add order's payment.
- `Update And Remove Order Payment` checked means user can update/remove order's payment.
