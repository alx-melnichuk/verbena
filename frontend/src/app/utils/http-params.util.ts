import { HttpParams } from '@angular/common/http';

export class HttpParamsUtil {

    public static create(data: any): HttpParams {
        let result: HttpParams = new HttpParams();
        const innData = (data || {});
        const keys: string[] = Object.keys(innData);
        for (const key of keys) {
            const value = innData[key];
            if (value != null) {
                result = result.set(key, value.toString());
            }
        }
        return result;
    }

    public static getParams(data: any): string {
        const result: string[] = [];
        const innData = (data || {});
        const keys: string[] = Object.keys(innData);
        for (const key of keys) {
            const value = innData[key];
            result.push(key + '=' + (typeof value === 'object' ? JSON.stringify(value) : value.toString()));
        }
        return result.join('&');
    }

}
