-- Migration: Add Two-Factor Authentication support
-- Adds two_factor_enabled to users and creates user_2fa table

-- Add two_factor_enabled column to users
ALTER TABLE `users` ADD COLUMN `two_factor_enabled` integer DEFAULT false;
--> statement-breakpoint
-- Add role_id column to users (for dynamic roles)
ALTER TABLE `users` ADD COLUMN `role_id` integer REFERENCES `roles`(`id`) ON DELETE SET NULL;
--> statement-breakpoint
-- Create user_2fa table for TOTP secrets and backup codes
CREATE TABLE `user_2fa` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`user_id` integer NOT NULL UNIQUE,
	`secret` text NOT NULL,
	`backup_codes` text NOT NULL,
	`backup_codes_remaining` integer DEFAULT 10,
	`created_at` integer,
	`updated_at` integer,
	FOREIGN KEY (`user_id`) REFERENCES `users`(`id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
CREATE INDEX `user_2fa_user_idx` ON `user_2fa` (`user_id`);
