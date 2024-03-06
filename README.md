# What is this
`Elerp` is a warehouse management system (WMS)/inventory system. The backend uses Rust + Sqlite, and frontend uses Vue + NaiveUI/ElementPlus. I'm not a professional programmer, I just do it for my job.

**Again, I'm not a professional programmer and anyone who wants to contribute code is more than welcome.**

# Install
- Docker compose file:
  ```
  version: "3.9"
  services:
    elerp:
      image: jinker25/elerp:latest
      container_name: elerp
      user: 1000:1000
      ports:
        - "3344:3344"
      restart: unless-stopped
      volumes:
        - "<your_data_folder_path>:/data"
  ```
  `sudo docker compose up -d` will start and listen port 3344. Default username `admin`, password `admin123`
  
# Web system
- No need to pay for any computer hardware!
- Visit the website directly to use it!

# Multi-language
- Support English, Chinese, Malay!

# Stock
- Multiple warehouse support!
- Check your inventory easily!
- Support filtering!

# Order
- Manage your orders and automatically update inventory!
- Support statistics. Check out your top 10 most popular products by sales and their total sales!
- Easily check if the quantity of items in your order is satisfactory!
- Support barcode scanner.
- Customers always place orders via messages? Supports adding multiple order items and automatically detecting messages!

# Guest order
- Do you want customers to add orders themselves, but don't want to share your account? Support adding guest orders, just share the link to customers, no account is required to access and place orders!
