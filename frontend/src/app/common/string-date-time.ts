/** Date value in ISO 8601 format 'yyyy-MM-dd' */
export type StringDate = string;

/** Date and time value in ISO 8601 format
 * 
 * 'yyyy-MM-ddThh:mm:ss.000Z', 'yyyy-MM-ddThh:mm:ssZ' */
export type StringDateTime = string;
// (new Date()).toISOString() '2020-02-08T10:20:30.000Z'
// (new Date()).toJSON()      '2020-02-08T10:20:30.000Z'
