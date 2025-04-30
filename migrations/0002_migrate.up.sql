-- Add up migration script here
CREATE TABLE IF NOT EXISTS ingredients (
  recipe_id VARCHAR(200) NOT NULL,
  ingredient_amount VARCHAR(200) NOT NULL,
  FOREIGN KEY (recipe_id) REFERENCES recipes(id)
);
