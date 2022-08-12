DROP TABLE IF EXISTS shopify_customer_orders CASCADE;
DROP TABLE IF EXISTS shopify_customer_addresses CASCADE;
DROP TABLE IF EXISTS shopify_contacts CASCADE;
DROP TABLE IF EXISTS generic_contacts CASCADE;
DROP TABLE IF EXISTS tag_contacts CASCADE;
DROP TABLE IF EXISTS tag_name CASCADE;
DROP TABLE IF EXISTS contacts;
 
CREATE TABLE IF NOT EXISTS contacts (
    id SERIAL PRIMARY KEY, 
    user_id TEXT NOT NULL UNIQUE, 
    phone TEXT, 
    email TEXT
);

CREATE TABLE IF NOT EXISTS generic_contacts (
    id SERIAL PRIMARY KEY, 
    identifier TEXT NOT NULL UNIQUE,
    user_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    name TEXT,
    photo TEXT,
    phone_numbers TEXT[],
    email_addresses TEXT[],
    FOREIGN KEY(user_id)
        REFERENCES contacts(user_id)
);

CREATE TABLE IF NOT EXISTS tag_name (
    id uuid DEFAULT uuid_generate_v4(),
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS tag_contacts (
    id SERIAL PRIMARY KEY, 
    user_id TEXT NOT NULL,
    tag_id uuid NOT NULL,
    identifier TEXT NOT NULL,
    FOREIGN KEY(user_id)
        REFERENCES contacts(user_id),
    FOREIGN KEY(tag_id)
        REFERENCES tag_name(id)
);

CREATE TABLE IF NOT EXISTS shopify_contacts (
    shopify_contacts_id SERIAL PRIMARY KEY,
    customer_id BIGINT NOT NULL UNIQUE,
    user_id TEXT NOT NULL, 
    customer_email TEXT NOT NULL,
    accepts_marketing BOOLEAN NOT NULL,
    customer_phone TEXT NOT NULL,
    note TEXT,
    currency TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    orders_count INT NOT NULL,
    total_spent TEXT NOT NULL,
    last_order_id BIGINT,
    verified_email BOOLEAN NOT NULL,
    tax_exempt BOOLEAN NOT NULL,
    FOREIGN KEY(user_id)
        REFERENCES contacts(user_id)
);

CREATE TABLE IF NOT EXISTS shopify_customer_orders (
    shopify_order_id SERIAL PRIMARY KEY,
    order_id BIGINT NOT NULL,
    order_customer_id BIGINT NOT NULL,
    app_id BIGINT NOT NULL,
    browser_ip TEXT,
    buyer_accepts_marketing BOOLEAN,
    cancel_reason TEXT,
    cancelled_at TIMESTAMP,
    cart_token TEXT,
    checkout_id BIGINT,
    closed_at TIMESTAMP,
    confirmed BOOLEAN NOT NULL,
    contact_email TEXT NOT NULL,
    order_created_at TIMESTAMP,
    order_currency TEXT NOT NULL,
    current_subtotal_price TEXT NOT NULL,
    current_total_discounts TEXT NOT NULL,
    current_total_duties_set TEXT,
    current_total_price TEXT NOT NULL,
    current_total_tax TEXT NOT NULL,
    device_id TEXT,
    order_email TEXT NOT NULL,
    financial_status TEXT NOT NULL,
    fulfillment_status TEXT,
    order_name TEXT NOT NULL,
    customer_number INT NOT NULL,
    order_number INT NOT NULL,
    order_note TEXT,
    processed_at TIMESTAMP NOT NULL,
    processing_method TEXT NOT NULL,
    subtotal_price TEXT NOT NULL,
    total_price TEXT NOT NULL,
    total_tax TEXT NOT NULL,
    total_price_usd TEXT NOT NULL,
    order_updated_at TIMESTAMP NOT NULL,
    line_items TEXT NOT NULL,
    FOREIGN KEY(order_customer_id)
        REFERENCES shopify_contacts(customer_id)
);

CREATE TABLE IF NOT EXISTS shopify_customer_addresses (
    address_id SERIAL PRIMARY KEY,
    address_customer_id BIGINT NOT NULL,
    address1 TEXT NOT NULL,
    address2 TEXT,
    city TEXT NOT NULL,
    country TEXT NOT NULL,
    country_code TEXT NOT NULL,
    country_name TEXT NOT NULL,
    company TEXT,
    province TEXT NOT NULL,
    province_code TEXT NOT NULL,
    zip TEXT NOT NULL,    
    FOREIGN KEY(address_customer_id)
        REFERENCES shopify_contacts(customer_id)
);