export type StringDate = string;
export type StringDateTime = string;   

export class StringDateTimeUtil {
  public static toISO(val: Date): StringDateTime {
    return val.toISOString();
  }
  public static toISODate(val: Date): StringDate {
    return val.toISOString().slice(0,10);
  }
  public static to_date(val: StringDateTime | null | undefined): Date | null {
    if (val == null || val ==undefined) {
      return null;
    }
    if (val.length != 20 && val.length != 24) {
      console.error(`The length of the string "${val}" is not 20 or 24.`);
      return null;
    }
    if (val[4] != '-' || val[7] != '-' || val[10] != 'T' || val[13] != ':' || val[16] != ':') {
      console.error(`The value '${val}' does not match the datetime format 'yyyy-MM-ddThh:mm:ss.000Z'`);
      return null;
    }
    return new Date(val);
  }
}
