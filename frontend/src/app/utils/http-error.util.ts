export interface ErrMsgObj {
    msg: string,
    obj: unknown,
}

export class HttpErrorUtil {
    public static mapErrMsgObj(status: number, error: any): ErrMsgObj | null {
        let msg: string = "";
        let obj: unknown | null = null;
        if (typeof error == "object") {
            // Extract the first value up to the ";" delimiter.
            const message = (error["message"] || "").split(";")[0];
            msg = !!message ? `${status + "." + message}` : "";
            const params = error["params"];
            obj = !!params ? { ...params } : null;
        } else {
            msg = (error || "").toString();
        }
        return !!msg ? { msg, obj } : null;
    }
    public static mapErrMsgObjs(status: number, error: any): ErrMsgObj[] {
        const result: ErrMsgObj[] = [];
        const errResList = (Array.isArray(error) ? error : (!error ? [] : [error]));
        for (let index = 0; index < errResList.length; index++) {
            const value = this.mapErrMsgObj(status, errResList[index]);
            if (!!value) {
                result.push(value);
            }
        }
        return result;
    }


    public static covertToMsg(status: number, error: any): string {
        let result: string = "";
        if (typeof error == "object") {
            // Extract the first value up to the ";" delimiter.
            const message = (error["message"] || "").split(";")[0];
            result = !!message ? `${status + "." + message}` : "";
        } else {
            result = (error || "").toString();
        }
        return result;
    }
}
