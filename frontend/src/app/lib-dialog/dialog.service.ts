import { ComponentType } from '@angular/cdk/portal';
import { Injectable } from '@angular/core';
import { MatDialog, MatDialogConfig } from '@angular/material/dialog';

import { ConfirmationComponent, ConfirmationData } from './confirmation/confirmation.component';

@Injectable({
  providedIn: 'root',
})
export class DialogService {
  constructor(private dialog: MatDialog) {}

  // ** Public API **

  public openConfirmation(message: string, title?: string, 
    params?: { btnNameCancel?: string | null, btnNameAccept?: string | null},
    dialogConfig?: MatDialogConfig
  ): Promise<unknown> {
    const confirmationData: ConfirmationData = { title, message };
    if (!!params?.btnNameCancel) {
      confirmationData.btnNameCancel = params.btnNameCancel;
    }
    if (!!params?.btnNameAccept) {
      confirmationData.btnNameAccept = params.btnNameAccept;
    }
    return this.openComponentExt(ConfirmationComponent, confirmationData, dialogConfig);
  }

  public openComponent(component: ComponentType<unknown>, dataParams: any): void {
    const dialogConfig = new MatDialogConfig();
    // Custom class for the overlay pane.
    dialogConfig.panelClass = ['app-modal-panel', 'large'];
    // Whether the dialog has a backdrop.
    dialogConfig.hasBackdrop = true;
    // Custom class for the backdrop.
    dialogConfig.backdropClass = 'app-modal-backdrop';
    // Whether the user can use escape or clicking on the backdrop to close the modal. disableClose?: boolean;
    dialogConfig.disableClose = false;
    dialogConfig.data = dataParams;

    this.dialog.open(component, dialogConfig);
  }

  public openComponentExt(component: ComponentType<unknown>, dataParams: any, dialogConfig?: MatDialogConfig): Promise<unknown> {
    const dialogCfg = {...(new MatDialogConfig()), ...dialogConfig };
    // Custom class for the overlay pane.
    dialogCfg.panelClass = ['app-modal-panel', 'large'];
    // Whether the dialog has a backdrop.
    dialogCfg.hasBackdrop = true;
    // Custom class for the backdrop.
    dialogCfg.backdropClass = 'app-modal-backdrop';
    // Whether the user can use escape or clicking on the backdrop to close the modal. disableClose?: boolean;
    dialogCfg.disableClose = false;
    dialogCfg.data = dataParams;

    const dialogRef = this.dialog.open(ConfirmationComponent, dialogCfg);
    // const answer = new Promise((resolve, reject) => {
    //   dialogRef.afterClosed().subscribe(result => {
    //     if (!!result) { resolve(result); } else { reject(); }
    //   });
    // });
    // return answer;
    return dialogRef.afterClosed().toPromise();
  }
}
