import { HttpErrorResponse } from '@angular/common/http';
import { TranslateService } from '@ngx-translate/core';

export class HttpErrorUtil {
    private static translate: TranslateService | undefined;
    public static setTranslate(translate: TranslateService | undefined): void {
        HttpErrorUtil.translate = translate;
    }
    public static getTranslate(): TranslateService | undefined {
        return HttpErrorUtil.translate;
    }
    public static getMsgs(errRes: HttpErrorResponse): string[] {
        let result: string[] = [];
        if (!!errRes && !!errRes.error) {
            const errResList = !Array.isArray(errRes.error) ? [errRes.error] : errRes.error;
            for (let index = 0; index < errResList.length; index++) {
                let value = '';
                const appError = errResList[index];
                if (typeof appError == 'object') {
                    const code = appError['code'] || '';
                    // Extract the first value up to the ";" delimiter.
                    const message = (appError['message'] || '').split(';')[0];
                    if (!!code) {
                        const key = `${code}${!!message ? '.' + message : ''}`;
                        const value2 = HttpErrorUtil.translate?.instant(key, appError['params'] || {}) || key;
                        value = value2 != key ? value2 : `${code}${!!message ? ': ' + message : ''}`;
                    }
                } else {
                    value = (appError || '').toString();
                }
                if (!!value) {
                    result.push(value);
                }
            }
        }
        if (result.length == 0 && !!HttpErrorUtil.translate) {
            const txt = errRes.status > 299 ? ` ${errRes.status} ${errRes.statusText}` : ``;
            result.push(HttpErrorUtil.translate.instant('error.server_api_call') + txt);
        }
        return result;
    }
    public static getMsg(error: any, statusText?: string | undefined): string[] {
        const result: string[] = [];
        if (!!error) {
            const errResList = !Array.isArray(error) ? [error] : error;
            for (let index = 0; index < errResList.length; index++) {
                let value = '';
                const appError = errResList[index];
                if (typeof appError == 'object') {
                    const code = appError['code'] || '';
                    // Extract the first value up to the ";" delimiter.
                    const message = (appError['message'] || '').split(';')[0];
                    if (!!code) {
                        const key = `${code}${!!message ? '.' + message : ''}`;
                        const value2 = HttpErrorUtil.translate?.instant(key, appError['params'] || {}) || key;
                        value = value2 != key ? value2 : `${code}${!!message ? ': ' + message : ''}`;
                    }
                } else {
                    value = (appError || '').toString();
                }
                if (!!value) {
                    result.push(value);
                }
            }
        }
        if (result.length == 0 && !!HttpErrorUtil.translate) {
            const txt = statusText != null ? ` ${statusText}` : ``;
            result.push(HttpErrorUtil.translate.instant('error.server_api_call') + txt);
        }
        return result;
    }
}
