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

  public openConfirmation(message: string, title?: string, btnNameCancel?: string | null, btnNameAccept?: string | null): Promise<unknown> {
    const confirmationData: ConfirmationData = { title, message };
    if (!!btnNameCancel) {
      confirmationData.btnNameCancel = btnNameCancel;
    }
    if (!!btnNameAccept) {
      confirmationData.btnNameAccept = btnNameAccept;
    }
    return this.openConfirmationExt(confirmationData);
  }

  public openConfirmationExt(confirmationData: ConfirmationData): Promise<unknown> {
    const dialogConfig = new MatDialogConfig();
    dialogConfig.data = {
      // Custom class for the overlay pane.
      panelClass: ['app-modal', 'large'],
      // Whether the dialog has a backdrop.
      hasBackdrop: true,
      // Custom class for the backdrop.
      backdropClass: 'app-modal-backdrop',
      // Whether the user can use escape or clicking on the backdrop to close the modal. disableClose?: boolean;
      disableClose: false,
      ...confirmationData,
    };
    const dialogRef = this.dialog.open(ConfirmationComponent, dialogConfig);
    // const answer = new Promise((resolve, reject) => {
    //   dialogRef.afterClosed().subscribe(result => {
    //     if (!!result) { resolve(result); } else { reject(); }
    //   });
    // });
    // return answer;
    return dialogRef.afterClosed().toPromise();
  }

  public openComponent(component: ComponentType<unknown>, dataParams: any): void {
    const dialogConfig = new MatDialogConfig();
    dialogConfig.data = {
      // Custom class for the overlay pane.
      panelClass: ['app-modal', 'large'],
      // Whether the dialog has a backdrop.
      hasBackdrop: true,
      // Custom class for the backdrop.
      backdropClass: 'app-modal-backdrop',
      // Whether the user can use escape or clicking on the backdrop to close the modal. disableClose?: boolean;
      disableClose: false,
      ...{ ...dataParams },
    };
    this.dialog.open(component, dialogConfig);
  }

  public openComponentExt(component: ComponentType<unknown>, dataParams: any): Promise<unknown> {
    const dialogConfig = new MatDialogConfig();
    dialogConfig.data = {
      // Custom class for the overlay pane.
      panelClass: ['app-modal', 'large'],
      // Whether the dialog has a backdrop.
      hasBackdrop: true,
      // Custom class for the backdrop.
      backdropClass: 'app-modal-backdrop',
      // Whether the user can use escape or clicking on the backdrop to close the modal. disableClose?: boolean;
      disableClose: false,
      ...{ ...dataParams },
    };
    return this.dialog.open(component, dialogConfig).afterClosed().toPromise();
  }
}
