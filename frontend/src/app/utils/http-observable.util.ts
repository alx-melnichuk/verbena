import { HttpErrorResponse } from '@angular/common/http';
import { Observable } from 'rxjs';
import { first } from 'rxjs/operators';

export class HttpObservableUtil {
    public static toPromise<T>(observable: Observable<T | HttpErrorResponse>): Promise<T | HttpErrorResponse> {
        if (!observable) {
            return Promise.reject(new HttpErrorResponse({ error: 'The value of "observable" is not defined.' }));
        }
        return new Promise<T | HttpErrorResponse>((resolve: (value: T) => void, reject: (reason: HttpErrorResponse) => void) => {
            observable.pipe(first()).subscribe({
                next: (response: T | HttpErrorResponse) => resolve(response as T),
                error: (err) => reject(new HttpErrorResponse(err)),
            });
        });
    }
}
