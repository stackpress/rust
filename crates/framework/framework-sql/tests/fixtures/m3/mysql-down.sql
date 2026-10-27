DROP INDEX `entry_identity` ON `entry`;
ALTER TABLE `entry` DROP COLUMN `summary`;
ALTER TABLE `entry` RENAME COLUMN `heading` TO `name`;
