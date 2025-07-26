-- Comprehensive MySQL Compatibility Test
-- Tests various MySQL features that should now work with PostMyRustache

-- Test 1: Basic table creation with AUTO_INCREMENT
CREATE TABLE test_users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(50) NOT NULL,
    email VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Test 2: Insert data
INSERT INTO test_users (username, email) VALUES 
('john_doe', 'john@example.com'),
('jane_smith', 'jane@example.com'),
('bob_wilson', 'bob@example.com');

-- Test 3: Basic SELECT queries
SELECT * FROM test_users;
SELECT COUNT(*) FROM test_users;
SELECT username, email FROM test_users WHERE id > 1;

-- Test 4: UPDATE queries  
UPDATE test_users SET email = 'john.doe@newdomain.com' WHERE username = 'john_doe';

-- Test 5: MySQL string functions (should be translated)
SELECT 
    username,
    UPPER(username) as upper_name,
    LENGTH(username) as name_length,
    CONCAT(username, '@company.com') as company_email
FROM test_users;

-- Test 6: Date functions (should be translated)
SELECT 
    username,
    created_at,
    NOW() as current_time,
    CURDATE() as current_date
FROM test_users;

-- Test 7: Test with backticks (should be translated to double quotes)
SELECT `username`, `email` FROM `test_users` WHERE `id` = 1;

-- Test 8: Test JOIN operations
CREATE TABLE test_posts (
    id INT AUTO_INCREMENT PRIMARY KEY,
    user_id INT,
    title VARCHAR(200),
    content TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO test_posts (user_id, title, content) VALUES
(1, 'First Post', 'This is the content of the first post'),
(2, 'Second Post', 'This is the content of the second post'),
(1, 'Another Post', 'More content from user 1');

-- Test 9: JOIN query
SELECT 
    u.username,
    p.title,
    p.content,
    p.created_at
FROM test_users u
INNER JOIN test_posts p ON u.id = p.user_id
ORDER BY p.created_at DESC;

-- Test 10: LEFT JOIN with aggregation
SELECT 
    u.username,
    u.email,
    COUNT(p.id) as post_count
FROM test_users u
LEFT JOIN test_posts p ON u.id = p.user_id
GROUP BY u.id, u.username, u.email
ORDER BY post_count DESC;

-- Test 11: Subquery
SELECT 
    username,
    email
FROM test_users 
WHERE id IN (
    SELECT DISTINCT user_id 
    FROM test_posts 
    WHERE title LIKE '%Post%'
);

-- Test 12: CASE statement
SELECT 
    username,
    COUNT(p.id) as post_count,
    CASE 
        WHEN COUNT(p.id) = 0 THEN 'No posts'
        WHEN COUNT(p.id) = 1 THEN 'One post'
        ELSE 'Multiple posts'
    END as post_status
FROM test_users u
LEFT JOIN test_posts p ON u.id = p.user_id
GROUP BY u.id, u.username;

-- Test 13: DELETE operation
DELETE FROM test_posts WHERE title = 'Second Post';

-- Test 14: Verify deletion
SELECT COUNT(*) as remaining_posts FROM test_posts;

-- Test 15: Clean up
DROP TABLE IF EXISTS test_posts;
DROP TABLE IF EXISTS test_users;