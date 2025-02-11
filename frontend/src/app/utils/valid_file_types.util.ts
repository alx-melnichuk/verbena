export class ValidFileTypesUtil {
    public static sorting(validTypes: string | null | undefined): string[] {
        const map: { [key: string]: string } = {};
        const validTypeList: string[] = (validTypes || '').split(',');
        for (let idx = 0; idx < validTypeList.length; idx++) {
            let value: string = validTypeList[idx].trim();
            if (value.length == 0) {
                continue;
            }
            if (map[value] == null) {
                map[value] = '1';
            }
        }
        return Object.keys(map);
    }
    public static text(validTypes: string | null | undefined): string[] {
        const map: { [key: string]: string } = {};
        const maket1 = 'image/';
        const validTypeList = this.sorting(validTypes);
        for (let idx = 0; idx < validTypeList.length; idx++) {
            let value = validTypeList[idx];
            if (value.startsWith(maket1)) {
                value = value.slice(maket1.length);
            }
            if (value.startsWith('.')) {
                value = value.slice(1);
            }
            if (map[value] == null) {
                map[value] = '1';
            }
        }
        return Object.keys(map);
    }
    public static checkFileByAccept(fileName: string, fileType: string, accept: string): boolean {
        if (!fileName || !fileType) {
            return false;
        }
        let isExistsInAvailable = false;
        const acceptList: string[] = ValidFileTypesUtil.sorting(accept);
        for (let idx = 0; idx < acceptList.length && !isExistsInAvailable; idx++) {
            const value = acceptList[idx];
            if (value[0] == '.') {
                isExistsInAvailable = fileName.endsWith(value);
            } else {
                isExistsInAvailable = (new RegExp(value)).test(fileType);
            }
        }
        return isExistsInAvailable;
    }
}  