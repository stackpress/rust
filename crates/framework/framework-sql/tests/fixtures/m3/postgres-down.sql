DROP INDEX "entry_identity";
ALTER TABLE "entry" DROP COLUMN "summary";
ALTER TABLE "entry" RENAME COLUMN "heading" TO "name";
