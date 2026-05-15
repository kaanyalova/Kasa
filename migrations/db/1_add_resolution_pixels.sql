ALTER TABLE  Image ADD COLUMN pixels INT;
UPDATE Image SET pixels = resolution_x * resolution_y;