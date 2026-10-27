ALTER TABLE "entry" ADD COLUMN "summary" TEXT;
ALTER TABLE "entry" RENAME COLUMN "name" TO "heading";
CREATE INDEX "entry_identity" ON "entry" ("id");
