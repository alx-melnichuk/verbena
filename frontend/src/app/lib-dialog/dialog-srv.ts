import { ComponentType } from "@angular/cdk/overlay";
import { inject, Injectable } from "@angular/core";
import { MatDialog, MatDialogConfig } from "@angular/material/dialog";
import { TranslateService } from "@ngx-translate/core";
import { firstValueFrom } from "rxjs";
import { ConfirmationData, Confirmation } from "./confirmation/confirmation";

@Injectable({
    providedIn: "root",
})
export class DialogSrv {
    private dialog: MatDialog = inject(MatDialog);
    private translateSrv: TranslateService = inject(TranslateService);

    // ** Public API **

    public openConfirmation(message: string, title?: string,
        params?: { btnNameCancel?: string | null, btnNameAccept?: string | null },
        dialogConfig?: MatDialogConfig
    ): Promise<unknown> {
        const confirmationData: ConfirmationData = { title, message };
        if (!!params?.btnNameCancel) {
            confirmationData.btnNameCancel = params.btnNameCancel;
        }
        if (!!params?.btnNameAccept) {
            confirmationData.btnNameAccept = params.btnNameAccept;
        }
        const dialogCnfg = dialogConfig || {};

        const panelClass = dialogCnfg.panelClass || [];
        const panelClass2 = Array.isArray(panelClass) ? panelClass : (!!panelClass ? [panelClass] : []);
        dialogCnfg.panelClass = panelClass2.concat(["app-modal-confirmation"]);

        return this.openComponent(Confirmation, confirmationData, dialogCnfg);
    }

    public openComponent(component: ComponentType<unknown>, dataParams: any, dialogConfig?: MatDialogConfig): Promise<unknown> {
        const dialogCfg = { ...(new MatDialogConfig()), ...(dialogConfig ?? {}) };
        // Custom class for the overlay pane.
        dialogCfg.panelClass = ["app-modal-panel", "large"].concat(dialogCfg.panelClass || []);
        // Whether the dialog has a backdrop.
        dialogCfg.hasBackdrop = true;
        // Custom class for the backdrop.
        // Note: Adding "backdropClass" overrides the default background style "cdk-overlay-dark-backdrop".
        // dialogCfg.backdropClass = "app-modal-backdrop";
        // Whether the user can use escape or clicking on the backdrop to close the modal. disableClose?: boolean;
        dialogCfg.disableClose = false;
        if (dataParams != null) {
            const prm: ConfirmationData = dataParams as ConfirmationData;
            if (!!prm.title) {
                prm.title = this.translateSrv.instant(prm.title);
            }
            if (!!prm.message) {
                prm.message = this.translateSrv.instant(prm.message);
            }
            if (!!prm.btnNameCancel) {
                prm.btnNameCancel = this.translateSrv.instant(prm.btnNameCancel);
            }
            if (!!prm.btnNameAccept) {
                prm.btnNameAccept = this.translateSrv.instant(prm.btnNameAccept);
            }
            dialogCfg.data = { ...(dialogCfg.data || {}), ...dataParams };
        }

        this.removeActiveFocus();
        const dialogRef = this.dialog.open(component, dialogCfg);
        // const answer = new Promise((resolve, reject) => {
        //   dialogRef.afterClosed().subscribe(result => {
        //     if (!!result) { resolve(result); } else { reject(); }
        //   });
        // });
        // return answer;
        // return dialogRef.afterClosed().toPromise();
        return firstValueFrom(dialogRef.afterClosed());
    }

    /** Remove active focus from the document */
    private removeActiveFocus() {
        if (document.activeElement instanceof HTMLElement) {
            document.activeElement.blur();
        }
    }
}
