import { Injectable } from "@angular/core";
import {
    MatSnackBar, MatSnackBarConfig, MatSnackBarHorizontalPosition, MatSnackBarRef, MatSnackBarVerticalPosition,
} from "@angular/material/snack-bar";
import { firstValueFrom } from "rxjs";
import { AlertMode, AlertDurationByMode } from "./alert/alert-info";
import { AlertWrap } from "./alert-wrap/alert-wrap";

@Injectable({
    providedIn: "root",
})
export class AlertSrv {
    private currentSnackBarRef: MatSnackBarRef<AlertWrap> | null = null;

    constructor(private snackBar: MatSnackBar) { }

    /**
     * Display title and message in toaster.
     * @param toasterMode ToasterMode — The message type for the toaster.
     * @param message string — The message to show in the toaster.
     * @param title string — The title to show in the toaster.
     * @param config MatSnackBarConfig<any> — Additional configuration options for the snackbar.
     * @returns MatSnackBarRef<ToasterComponent>
     */
    public show(
        toasterMode: AlertMode,
        message: string,
        title?: string,
        config?: MatSnackBarConfig<any>
    ): MatSnackBarRef<AlertWrap> {
        if (this.currentSnackBarRef != null) {
            this.currentSnackBarRef.dismiss();
        }
        const mode: AlertMode = toasterMode || AlertMode.comment;
        const duration = AlertDurationByMode[mode];
        const horizontalPosition: MatSnackBarHorizontalPosition = "center"; // ["start" | "center" | "end" | "left" | "right"]
        const verticalPosition: MatSnackBarVerticalPosition = "bottom"; // ["top" | "bottom"]

        const innConfig: MatSnackBarConfig<any> = {
            ...{ duration, horizontalPosition, verticalPosition },
            ...(config || {}),
            ...{ panelClass: ["app-alert-wrap-panel", "app-" + mode] },
            ...{ data: { mode, title, message } },
        };
        this.currentSnackBarRef = this.snackBar.openFromComponent(AlertWrap, innConfig);
        firstValueFrom(this.currentSnackBarRef.afterDismissed())
            .then(() => {
                this.currentSnackBarRef = null;
            });
        return this.currentSnackBarRef;
    }

    public showComment(message: string, title?: string): MatSnackBarRef<AlertWrap> {
        return this.show(AlertMode.comment, message, title);
    }

    public showInfo(message: string, title?: string): MatSnackBarRef<AlertWrap> {
        return this.show(AlertMode.info, message, title);
    }

    public showWarning(message: string, title?: string): MatSnackBarRef<AlertWrap> {
        return this.show(AlertMode.warning, message, title);
    }

    public showError(message: string, title?: string): MatSnackBarRef<AlertWrap> {
        return this.show(AlertMode.error, message, title);
    }

    public showSuccess(message: string, title?: string): MatSnackBarRef<AlertWrap> {
        return this.show(AlertMode.success, message, title);
    }

    public hide(): void {
        if (this.currentSnackBarRef != null) {
            this.currentSnackBarRef.dismiss();
        }
    }
}
