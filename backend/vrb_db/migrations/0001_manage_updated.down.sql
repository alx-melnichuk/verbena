-- Remove the system function "set_updated_at()".

-- **

/* Remove the "manage_updated_at" function. */
DROP FUNCTION IF EXISTS manage_updated_at(_tbl regclass);
/* Remove the "set_updated_at" function. */
DROP FUNCTION IF EXISTS set_updated_at();

-- **
