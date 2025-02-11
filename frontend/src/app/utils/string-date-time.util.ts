import { StringDateTime } from "../common/string-date-time";

/*
let d1 = (new Date())
d1                      =>  Fri Oct 11 2020 14:35:21 GMT+0300 (Eastern European Summer Time)
d1.toDateString()       => 'Fri Oct 11 2020'
d1.toGMTString()        => 'Fri, 11 Oct 2020 11:35:21 GMT'
d1.toISOString()        => '2020-10-11T11:35:21.464Z'
d1.toJSON()             => '2020-10-11T11:35:21.464Z'
d1.toLocaleDateString() => '10/11/2020'
d1.toLocaleString()     => '10/11/2020, 2:35:21 PM'
d1.toLocaleTimeString() => '2:35:21 PM'
d1.toString()           => 'Fri Oct 11 2020 14:35:21 GMT+0300 (Eastern European Summer Time)'
d1.toTimeString()       => '14:35:21 GMT+0300 (Eastern European Summer Time)'
d1.toUTCString()        => 'Fri, 11 Oct 2020 11:35:21 GMT'
*/

export class StringDateTimeUtil {
    /** Converts from ISO 8601 format 'yyyy-MM-ddThh:mm:ss.000Z', 'yyyy-MM-ddThh:mm:ssZ' to date. */
    public static toDate(val: StringDateTime | null | undefined): Date | null {
        if (val == null || val == undefined) {
            return null;
        }
        if (val.length != 20 && val.length != 24) {
            console.error(`The length of the string "${val}" is not 20 or 24.`);
            return null;
        }
        const ln = val.length - 1;
        if (val[4] != '-' || val[7] != '-' || val[10] != 'T' || val[13] != ':' || val[16] != ':' || val[ln] != 'Z') {
            console.error(`The value '${val}' does not match the datetime format 'yyyy-MM-ddThh:mm:ss.000Z'`);
            return null;
        }
        return new Date(val);
    }
}
