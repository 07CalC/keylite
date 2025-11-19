#ifndef KEYLITE_H
#define KEYLITE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque database handle */
typedef struct KeyliteDb KeyliteDb;

/* Opaque iterator handle */
typedef struct KeyliteIterator KeyliteIterator;

/* Result codes */
typedef enum {
  KEYLITE_OK = 0,
  KEYLITE_ERR_NULL = 1,
  KEYLITE_ERR_IO = 2,
  KEYLITE_ERR_UTF8 = 3,
  KEYLITE_ERR_OTHER = 4
} KeyliteResult;

/* Create/open a database at the given path */
KeyliteResult keylite_open(const char *path, KeyliteDb **db_out);

/* Close and free a database handle */
void keylite_close(KeyliteDb *db);

/* Put a key-value pair into the database */
KeyliteResult keylite_put(KeyliteDb *db, const uint8_t *key, size_t key_len,
                          const uint8_t *val, size_t val_len);

/* Get a value from the database
 * Returns KEYLITE_OK with val_out=NULL if key not found
 * Returned value must be freed with keylite_free_value */
KeyliteResult keylite_get(KeyliteDb *db, const uint8_t *key, size_t key_len,
                          uint8_t **val_out, size_t *val_len_out);

/* Free a value returned from keylite_get */
void keylite_free_value(uint8_t *val, size_t len);

/* Delete a key from the database */
KeyliteResult keylite_del(KeyliteDb *db, const uint8_t *key, size_t key_len);

/* ============================================================================
 * String API - automatically handles UTF-8 encoding/decoding
 * Flush and compaction are handled automatically by the database
 * ============================================================================
 */

/* Put a string key-value pair into the database
 * Both key and val must be null-terminated UTF-8 strings */
KeyliteResult keylite_put_str(KeyliteDb *db, const char *key, const char *val);

/* Get a string value from the database
 * Returns KEYLITE_OK with val_out=NULL if key not found
 * Returned string is null-terminated and must be freed with keylite_free_str
 * Returns KEYLITE_ERR_UTF8 if stored value is not valid UTF-8 */
KeyliteResult keylite_get_str(KeyliteDb *db, const char *key, char **val_out);

/* Free a string returned from keylite_get_str */
void keylite_free_str(char *val);

/* Delete a key from the database (string version)
 * Key must be a null-terminated UTF-8 string */
KeyliteResult keylite_del_str(KeyliteDb *db, const char *key);

/* ============================================================================
 * Scan API - iterate over key-value pairs
 * ============================================================================
 */

/* Create an iterator for scanning a range of keys
 * start and end can be NULL to scan from beginning/to end
 * The iterator must be freed with keylite_iter_free */
KeyliteResult keylite_scan(KeyliteDb *db, const uint8_t *start,
                           size_t start_len, const uint8_t *end, size_t end_len,
                           KeyliteIterator **iter_out);

/* Create an iterator for scanning a range of string keys
 * start and end can be NULL to scan from beginning/to end
 * start and end must be null-terminated UTF-8 strings
 * The iterator must be freed with keylite_iter_free */
KeyliteResult keylite_scan_str(KeyliteDb *db, const char *start,
                               const char *end, KeyliteIterator **iter_out);

/* Get the next key-value pair from the iterator
 * Returns KEYLITE_OK with key_out=NULL when iteration is complete
 * Returned key and value must be freed with keylite_free_value */
KeyliteResult keylite_iter_next(KeyliteIterator *iter, uint8_t **key_out,
                                size_t *key_len_out, uint8_t **val_out,
                                size_t *val_len_out);

/* Free an iterator */
void keylite_iter_free(KeyliteIterator *iter);

#ifdef __cplusplus
}
#endif

#endif /* KEYLITE_H */
