import { Injectable } from '@angular/core';
import { AlertDurationByMode, AlertMode } from './alert/alert.interface';
import { AlertWrapComponent } from './alert-wrap/alert-wrap.component';
import {
  MatSnackBar,
  MatSnackBarConfig,
  MatSnackBarHorizontalPosition,
  MatSnackBarRef,
  MatSnackBarVerticalPosition,
} from '@angular/material/snack-bar';

@Injectable({
  providedIn: 'root',
})
export class AlertService {
  private currentSnackBarRef: MatSnackBarRef<AlertWrapComponent> | null = null;

  constructor(private snackBar: MatSnackBar) {
    console.log(`AlertService() snackBar!=null : ${this.snackBar != null}`); // #-
  }

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
  ): MatSnackBarRef<AlertWrapComponent> {
    const mode: AlertMode = toasterMode || AlertMode.comment;
    const duration = AlertDurationByMode[mode];
    const horizontalPosition: MatSnackBarHorizontalPosition = 'center'; // ['start' | 'center' | 'end' | 'left' | 'right']
    const verticalPosition: MatSnackBarVerticalPosition = 'bottom'; // ['top' | 'bottom']

    const innConfig: MatSnackBarConfig<any> = {
      ...{ duration, horizontalPosition, verticalPosition },
      ...(config || {}),
      ...{ panelClass: ['app-alert-wrap-panel'] },
      ...{ data: { mode, title, message } },
    };
    this.currentSnackBarRef = this.snackBar.openFromComponent(AlertWrapComponent, innConfig);
    this.currentSnackBarRef
      .afterDismissed()
      .toPromise()
      .finally(() => {
        this.currentSnackBarRef = null;
      });
    return this.currentSnackBarRef;
  }

  public showComment(message: string, title?: string): MatSnackBarRef<AlertWrapComponent> {
    return this.show(AlertMode.comment, message, title);
  }

  public showInfo(message: string, title?: string): MatSnackBarRef<AlertWrapComponent> {
    return this.show(AlertMode.info, message, title);
  }

  public showWarning(message: string, title?: string): MatSnackBarRef<AlertWrapComponent> {
    return this.show(AlertMode.warning, message, title);
  }

  public showError(message: string, title?: string): MatSnackBarRef<AlertWrapComponent> {
    return this.show(AlertMode.error, message, title);
  }

  public showSuccess(message: string, title?: string): MatSnackBarRef<AlertWrapComponent> {
    return this.show(AlertMode.success, message, title);
  }

  public hide(): void {
    if (this.currentSnackBarRef != null) {
      this.currentSnackBarRef.dismiss();
    }
  }
}
