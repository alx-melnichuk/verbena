export class ValidFileTypesUtil {
  public static get(validTypes: string | null | undefined): string {
    const result: Array<string> = [];
    const validTypeList = (validTypes || '').split(',');
    for (let idx = 0; idx < validTypeList.length; idx++) {
        result.push(validTypeList[idx].replace('image/', ''));
    }
    return result.length > 0 ? result.join(', ') : '';
  }
}  