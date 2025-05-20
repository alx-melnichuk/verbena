import { ActivatedRouteSnapshot, CanDeactivateFn, RouterStateSnapshot } from '@angular/router';
import { Observable } from 'rxjs';


export interface IDeactivatePage {
    canExit: () => Observable<boolean> | Promise<boolean> | boolean;
}

export const deactivatePageGuard: CanDeactivateFn<unknown> = (
    component: unknown, currentRoute: ActivatedRouteSnapshot, currentState: RouterStateSnapshot, nextState: RouterStateSnapshot
) => {
    return (component as IDeactivatePage).canExit ? (component as IDeactivatePage).canExit() : true;
};
