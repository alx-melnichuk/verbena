export class TimeUtil {
  public static parseTimeHHMM(value: String | null | undefined): { hours: number, minutes: number } {
    let hours: number = 0;
    let minutes: number = 0;
    if (!!value && value.length > 4) {
        const hoursStr = value.slice(0,2);
        hours = parseInt(hoursStr, 10);
        const minutesStr = value.slice(3,6);
        minutes = parseInt(minutesStr, 10);
    }
    return { hours, minutes };
  }

}