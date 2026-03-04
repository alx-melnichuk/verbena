import { ActivatedRouteSnapshot, CanDeactivateFn, RouterStateSnapshot } from "@angular/router";
import { inject } from "@angular/core";
import { TranslateService } from "@ngx-translate/core";
import { DialogSrv } from "../lib-dialog/dialog-srv";

export class UnsavedDeActivateUtil {
    private static innConfirmText: string = "";
    public static setConfirmText(text: string): void {
        this.innConfirmText = text;
    }
    public static getConfirmText(): string {
        return this.innConfirmText;
    }
}
export interface HasUnsavedChanges {
    hasUnsavedChanges(): boolean;
}

export const unsavedDeActivatePageGuard: CanDeactivateFn<HasUnsavedChanges> = (
    component: unknown, _currentRoute: ActivatedRouteSnapshot, _currentState: RouterStateSnapshot, _nextState: RouterStateSnapshot
) => {
    const dialogSrv: DialogSrv = inject(DialogSrv);
    if ((component as HasUnsavedChanges)?.hasUnsavedChanges()) {
        const confirm_text = UnsavedDeActivateUtil.getConfirmText();
        const confirm_text2 = !!confirm_text ? inject(TranslateService).instant(confirm_text) : "";
        const message = confirm_text2 || "dialog.confirm_exit_without_saving";

        return dialogSrv.openConfirmation(message, "", { btnNameCancel: "buttons.no", btnNameAccept: "buttons.yes" })
            .then((value) => !!value)
            .catch(() => false);
    }
    return true;
};
