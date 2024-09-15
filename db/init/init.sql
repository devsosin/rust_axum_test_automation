CREATE TABLE tb_image(
    id SERIAL PRIMARY KEY,
    original_name VARCHAR(128),
    image_key VARCHAR(64)
);

CREATE TABLE tb_user(
    id SERIAL PRIMARY KEY,
    login_type VARCHAR(8) NOT NULL,
    username VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    access_token VARCHAR(255),
    
    nickname VARCHAR(12),
    profile_id INT,
    phone VARCHAR(25),
    email VARCHAR(255) NOT NULL,

    is_active BOOLEAN DEFAULT TRUE,
    is_admin BOOLEAN DEFAULT FALSE,

    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP,

    FOREIGN KEY (profile_id) REFERENCES tb_image(id)
);

CREATE TABLE tb_book_type(
    id SMALLSERIAL PRIMARY KEY,
    name VARCHAR(4) NOT NULL
);

CREATE TABLE tb_book(
    id SERIAL PRIMARY KEY,
    name VARCHAR(16) NOT NULL,
    type_id SMALLINT NOT NULL,

    FOREIGN KEY (type_id) REFERENCES tb_book_type(id)
);

CREATE TABLE tb_user_book_role(
    user_id INT NOT NULL,
    book_id INT NOT NULL,
    role VARCHAR(8),

    PRIMARY KEY (user_id, book_id)
);

CREATE TABLE tb_base_category(
    id SMALLSERIAL PRIMARY KEY,
    type_id SMALLINT NOT NULL,
    book_id INT,
    is_record BOOLEAN NOT NULL,
    name VARCHAR(16) NOT NULL,
    color VARCHAR(8) NOT NULL,

    FOREIGN KEY (book_id) REFERENCES tb_book(id),
    FOREIGN KEY (type_id) REFERENCES tb_book_type(id)
);

CREATE TABLE tb_sub_category(
    id SERIAL PRIMARY KEY,
    base_id SMALLINT NOT NULL,
    name VARCHAR(32) NOT NULL,

    FOREIGN KEY (base_id) REFERENCES tb_base_category(id)
);

CREATE TABLE tb_diary(
    id SERIAL PRIMARY KEY,
    book_id INT,
    title VARCHAR(50),
    content VARCHAR(500),

    target_dt TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP,

    FOREIGN KEY (book_id) REFERENCES tb_book(id)
);

CREATE TABLE tb_asset(
    id SERIAL PRIMARY KEY,
    book_id INT NOT NULL,
    sub_category_id INT NOT NULL,
    code VARCHAR(32),
    memo VARCHAR(16),
    balance BIGINT NOT NULL,

    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP,

    FOREIGN KEY (book_id) REFERENCES tb_book(id),
    FOREIGN KEY (sub_category_id) REFERENCES tb_sub_category(id)
);

CREATE TABLE tb_record(
    id BIGSERIAL PRIMARY KEY,
    book_id INT NOT NULL,
    sub_category_id INT NOT NULL,

    amount INTEGER NOT NULL,
    memo VARCHAR(32),

    asset_id INT,

    target_dt TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP,

    FOREIGN KEY (book_id) REFERENCES tb_book(id),
    FOREIGN KEY (asset_id) REFERENCES tb_asset(id),
    FOREIGN KEY (sub_category_id) REFERENCES tb_sub_category(id)
);

CREATE TABLE tb_connect(
    id SERIAL PRIMARY KEY,
    name VARCHAR(32) NOT NULL
);

CREATE TABLE tb_record_connect(
    record_id BIGINT,
    connect_id INT,

    PRIMARY KEY (record_id, connect_id)
);

INSERT INTO tb_book_type(name) VALUES
    ('개인'),
    ('커플'),
    ('기업');

INSERT INTO tb_book(name, type_id) VALUES
    ('테스트 가계부', 1);

INSERT INTO tb_base_category(type_id, is_record, name, color) VALUES
    (1, FALSE, '계좌', '000000'), -- 1
    (1, FALSE, '대출', '000000'), -- 2
    (1, FALSE, '증권', '000000'), -- 3
    (1, FALSE, '연금', '000000'), -- 4
    (1, FALSE, '포인트', '000000'), -- 5
    (1, FALSE, '기타', '000000'), -- 6
    (1, TRUE, '수입', 'F14567'), -- 7
    (1, TRUE, '고정지출', '4284E8'), -- 8
    (1, TRUE, '변동지출', '4284E8'), -- 9
    (1, TRUE, '저축', '000000'); -- 10

-- 뒷 부분은 좀 더 지식이 많아지면
    -- -- 커플
    -- -- 커플통장
    -- (2, FALSE, '현금', '000000'), -- 1
    -- -- 모으기, 사용
    -- (2, TRUE, '수입', '000000'), -- 6

    -- -- 기업
    -- (3, FALSE, '유동자산', '000000'), -- 1
    -- (3, FALSE, '고정자산', '000000'), -- 1

    -- (3, TRUE, '외상매입금', '000000'), -- 6
    -- (3, TRUE, '매입금', '000000'), -- 6
    -- (3, TRUE, '외상매출금', '000000'), -- 6
    -- (3, TRUE, '매출금', '000000'), -- 6

-- 현금을 신경써야할까?
-- 토스
-- 계좌 (입출금, 저축) 대출 증권(종합위탁, CMA, ISA - 개인종합자산관리계좌) 연금(확정기여형 DC, DP) 포인트 기타 (부동산 자동차 현금) 보험 
-- 일단 중요한 건 자산은 연관자산이 무엇이냐보다 어떻게 사용중이냐를 확인하고 싶은 것
INSERT INTO tb_sub_category(base_id, name) VALUES
    (1, '입출금'), -- 계좌명
    (1, '저축'), -- 계좌명
    (3, '종합위탁'),
    (3, 'CMA'),
    (3, 'ISA'),
    (6, '부동산'),
    (6, '자동차'),
    (6, '현금'),

    (7, '급여'), -- 회사
    (7, '사업'), 
    (7, '기타'),
    (8, '주거비'),
    (8, '통신비'),
    (8, '교통비'),
    (8, '공과금'), -- 전기요금, 가스요금, 수도요금
    (8, '구독료'),
    (9, '식비'),
    (9, '생필품비'),
    (9, '취미'),
    (9, '장비'),
    (9, '의류/미용비'),
    (9, '교육/문화비'),
    (9, '여행'),
    (9, '친목비'),
    (9, '의료비'),
    (9, '기타'),
    (9, '경조사'),
    (10, '적금');

-- 큰 카테고리만 있는 경우? 있으면 안될것같은데
-- 세부로 들어가면 주식 종목 등등으로 구분되어 빠질 것

INSERT INTO tb_connect(name) VALUES
    ('테스트 커넥트');

INSERT INTO tb_record (book_id, sub_category_id, amount, memo, target_dt, created_at, asset_id) 
        VALUES (1, 18, 15000, '감자탕', NOW(), NOW(), NULL);

INSERT INTO tb_user(login_type, username, password, email) VALUES
    ('email', 'test_user', 'test_password', 'test@test.test');