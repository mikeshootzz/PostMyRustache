-- MySQL Compatibility Test Suite
-- This script tests various MySQL data types and complex queries

-- Drop tables if they exist (for repeated testing)
DROP TABLE IF EXISTS order_items;
DROP TABLE IF EXISTS orders;
DROP TABLE IF EXISTS customers;
DROP TABLE IF EXISTS products;
DROP TABLE IF EXISTS categories;
DROP TABLE IF EXISTS test_datatypes;

-- Test table with comprehensive MySQL data types
CREATE TABLE test_datatypes (
    -- Integer types
    id INT AUTO_INCREMENT PRIMARY KEY,
    tiny_int_col TINYINT,
    small_int_col SMALLINT,
    medium_int_col MEDIUMINT,
    big_int_col BIGINT,
    
    -- Unsigned integers
    unsigned_int_col INT UNSIGNED,
    unsigned_big_int_col BIGINT UNSIGNED,
    
    -- Decimal and numeric types
    decimal_col DECIMAL(10,2),
    numeric_col NUMERIC(15,4),
    float_col FLOAT,
    double_col DOUBLE,
    
    -- String types
    char_col CHAR(10),
    varchar_col VARCHAR(255),
    text_col TEXT,
    medium_text_col MEDIUMTEXT,
    long_text_col LONGTEXT,
    
    -- Binary types
    binary_col BINARY(16),
    varbinary_col VARBINARY(255),
    blob_col BLOB,
    medium_blob_col MEDIUMBLOB,
    long_blob_col LONGBLOB,
    
    -- Date and time types
    date_col DATE,
    time_col TIME,
    datetime_col DATETIME,
    timestamp_col TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    year_col YEAR,
    
    -- Boolean type
    bool_col BOOLEAN,
    
    -- JSON type (MySQL 5.7+)
    json_col JSON,
    
    -- Enum and Set types
    enum_col ENUM('small', 'medium', 'large'),
    set_col SET('red', 'green', 'blue')
);

-- Create normalized test tables for JOIN testing
CREATE TABLE categories (
    category_id INT AUTO_INCREMENT PRIMARY KEY,
    category_name VARCHAR(100) NOT NULL,
    description TEXT
);

CREATE TABLE products (
    product_id INT AUTO_INCREMENT PRIMARY KEY,
    product_name VARCHAR(200) NOT NULL,
    category_id INT,
    price DECIMAL(10,2),
    stock_quantity INT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (category_id) REFERENCES categories(category_id)
);

CREATE TABLE customers (
    customer_id INT AUTO_INCREMENT PRIMARY KEY,
    first_name VARCHAR(50),
    last_name VARCHAR(50),
    email VARCHAR(100) UNIQUE,
    phone VARCHAR(20),
    address TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE orders (
    order_id INT AUTO_INCREMENT PRIMARY KEY,
    customer_id INT,
    order_date DATE,
    total_amount DECIMAL(10,2),
    status ENUM('pending', 'processing', 'shipped', 'delivered', 'cancelled'),
    FOREIGN KEY (customer_id) REFERENCES customers(customer_id)
);

CREATE TABLE order_items (
    order_item_id INT AUTO_INCREMENT PRIMARY KEY,
    order_id INT,
    product_id INT,
    quantity INT,
    unit_price DECIMAL(10,2),
    FOREIGN KEY (order_id) REFERENCES orders(order_id),
    FOREIGN KEY (product_id) REFERENCES products(product_id)
);

-- Insert test data
INSERT INTO categories (category_name, description) VALUES
('Electronics', 'Electronic devices and accessories'),
('Clothing', 'Apparel and fashion items'),
('Books', 'Books and educational materials');

INSERT INTO products (product_name, category_id, price, stock_quantity) VALUES
('Laptop Computer', 1, 999.99, 50),
('Smartphone', 1, 599.99, 100),
('T-Shirt', 2, 19.99, 200),
('Programming Book', 3, 49.99, 75);

INSERT INTO customers (first_name, last_name, email, phone, address) VALUES
('John', 'Doe', 'john.doe@email.com', '555-1234', '123 Main St'),
('Jane', 'Smith', 'jane.smith@email.com', '555-5678', '456 Oak Ave'),
('Bob', 'Johnson', 'bob.johnson@email.com', '555-9012', '789 Pine Rd');

INSERT INTO orders (customer_id, order_date, total_amount, status) VALUES
(1, '2024-01-15', 1049.98, 'delivered'),
(2, '2024-02-01', 599.99, 'shipped'),
(3, '2024-02-15', 69.98, 'processing');

INSERT INTO order_items (order_id, product_id, quantity, unit_price) VALUES
(1, 1, 1, 999.99),
(1, 3, 2, 19.99),
(2, 2, 1, 599.99),
(3, 3, 1, 19.99),
(3, 4, 1, 49.99);

-- Insert comprehensive data type test data
INSERT INTO test_datatypes (
    tiny_int_col, small_int_col, medium_int_col, big_int_col,
    unsigned_int_col, unsigned_big_int_col,
    decimal_col, numeric_col, float_col, double_col,
    char_col, varchar_col, text_col, medium_text_col, long_text_col,
    binary_col, varbinary_col, blob_col, medium_blob_col, long_blob_col,
    date_col, time_col, datetime_col, year_col,
    bool_col, json_col, enum_col, set_col
) VALUES (
    127, 32767, 8388607, 9223372036854775807,
    4294967295, 18446744073709551615,
    12345.67, 123456789.1234, 3.14159, 2.718281828,
    'CHAR_TEST', 'VARCHAR_TEST', 'TEXT_TEST', 'MEDIUMTEXT_TEST', 'LONGTEXT_TEST',
    UNHEX('0123456789ABCDEF0123456789ABCDEF'), 
    UNHEX('DEADBEEF'), 
    'BLOB_TEST', 'MEDIUMBLOB_TEST', 'LONGBLOB_TEST',
    '2024-01-15', '14:30:00', '2024-01-15 14:30:00', 2024,
    TRUE, '{"key": "value", "number": 42}', 'large', 'red,blue'
);

-- Test MySQL-specific functions and operators
SELECT 
    -- String functions
    CONCAT(first_name, ' ', last_name) AS full_name,
    UPPER(email) AS upper_email,
    LOWER(email) AS lower_email,
    LENGTH(first_name) AS name_length,
    SUBSTRING(email, 1, 5) AS email_prefix,
    
    -- Date functions
    NOW() AS current_timestamp,
    CURDATE() AS current_date,
    YEAR(created_at) AS created_year,
    MONTH(created_at) AS created_month,
    DAY(created_at) AS created_day,
    
    -- Math functions
    ROUND(999.99 * 1.08, 2) AS tax_included,
    ABS(-42) AS absolute_value,
    CEIL(19.1) AS ceiling_value,
    FLOOR(19.9) AS floor_value
FROM customers;

-- Test complex JOIN queries
SELECT 
    c.first_name,
    c.last_name,
    o.order_id,
    o.order_date,
    o.total_amount,
    o.status,
    oi.quantity,
    p.product_name,
    p.price,
    cat.category_name
FROM customers c
INNER JOIN orders o ON c.customer_id = o.customer_id
INNER JOIN order_items oi ON o.order_id = oi.order_id
INNER JOIN products p ON oi.product_id = p.product_id
INNER JOIN categories cat ON p.category_id = cat.category_id
ORDER BY c.last_name, o.order_date;

-- Test LEFT JOIN
SELECT 
    c.first_name,
    c.last_name,
    COUNT(o.order_id) AS order_count,
    COALESCE(SUM(o.total_amount), 0) AS total_spent
FROM customers c
LEFT JOIN orders o ON c.customer_id = o.customer_id
GROUP BY c.customer_id, c.first_name, c.last_name
ORDER BY total_spent DESC;

-- Test subqueries
SELECT 
    p.product_name,
    p.price,
    (SELECT AVG(price) FROM products) AS avg_price,
    p.price - (SELECT AVG(price) FROM products) AS price_diff
FROM products p
WHERE p.price > (SELECT AVG(price) FROM products);

-- Test EXISTS subquery
SELECT c.first_name, c.last_name, c.email
FROM customers c
WHERE EXISTS (
    SELECT 1 FROM orders o 
    WHERE o.customer_id = c.customer_id 
    AND o.status = 'delivered'
);

-- Test IN subquery
SELECT product_name, price
FROM products
WHERE category_id IN (
    SELECT category_id 
    FROM categories 
    WHERE category_name IN ('Electronics', 'Books')
);

-- Test aggregation functions
SELECT 
    cat.category_name,
    COUNT(p.product_id) AS product_count,
    AVG(p.price) AS avg_price,
    MIN(p.price) AS min_price,
    MAX(p.price) AS max_price,
    SUM(p.stock_quantity) AS total_stock
FROM categories cat
LEFT JOIN products p ON cat.category_id = p.category_id
GROUP BY cat.category_id, cat.category_name
HAVING COUNT(p.product_id) > 0
ORDER BY avg_price DESC;

-- Test UNION
SELECT 'Customer' AS type, first_name AS name, email
FROM customers
UNION
SELECT 'Product' AS type, product_name AS name, CAST(price AS CHAR) AS email
FROM products;

-- Test CASE statement
SELECT 
    product_name,
    price,
    CASE 
        WHEN price < 50 THEN 'Budget'
        WHEN price < 500 THEN 'Mid-range'
        ELSE 'Premium'
    END AS price_category,
    stock_quantity,
    CASE 
        WHEN stock_quantity = 0 THEN 'Out of Stock'
        WHEN stock_quantity < 20 THEN 'Low Stock'
        ELSE 'In Stock'
    END AS stock_status
FROM products;

-- Test window functions (MySQL 8.0+)
SELECT 
    c.first_name,
    c.last_name,
    o.order_date,
    o.total_amount,
    ROW_NUMBER() OVER (PARTITION BY c.customer_id ORDER BY o.order_date) AS order_number,
    SUM(o.total_amount) OVER (PARTITION BY c.customer_id) AS customer_total,
    RANK() OVER (ORDER BY o.total_amount DESC) AS amount_rank
FROM customers c
INNER JOIN orders o ON c.customer_id = o.customer_id;

-- Test UPDATE with JOIN
UPDATE products p
INNER JOIN categories c ON p.category_id = c.category_id
SET p.price = p.price * 1.10
WHERE c.category_name = 'Electronics';

-- Test DELETE with JOIN
DELETE oi FROM order_items oi
INNER JOIN orders o ON oi.order_id = o.order_id
WHERE o.status = 'cancelled';

-- Test MySQL-specific variables and functions
SELECT @@version_comment;
SELECT @@sql_mode;
SELECT CONNECTION_ID();
SELECT DATABASE();
SELECT USER();

-- Test LIMIT and OFFSET
SELECT product_name, price
FROM products
ORDER BY price DESC
LIMIT 2 OFFSET 1;

-- Test multiple table UPDATE
UPDATE customers c, orders o
SET c.address = 'Updated Address'
WHERE c.customer_id = o.customer_id
AND o.status = 'delivered';