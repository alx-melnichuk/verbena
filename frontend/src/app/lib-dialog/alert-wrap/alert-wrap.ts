import { CommonModule } from "@angular/common";
import { ChangeDetectionStrategy, Component, inject, ViewEncapsulation } from "@angular/core";
import { MatSnackBarModule, MatSnackBarRef, MAT_SNACK_BAR_DATA } from "@angular/material/snack-bar";
import { Alert } from "../alert/alert";
import { AlertInfo } from "../alert/alert-info";

@Component({
    selector: "app-alert-wrap",
    exportAs: "appAlertWrap",
    standalone: true,
    imports: [CommonModule, MatSnackBarModule, Alert],
    templateUrl: "./alert-wrap.html",
    styleUrl: "./alert-wrap.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class AlertWrap {
    public snackBarRef: MatSnackBarRef<AlertWrap> = inject(MatSnackBarRef<AlertWrap>);
    public data: AlertInfo = inject(MAT_SNACK_BAR_DATA);

    // Performs the close on the snack bar.
    public doClose(): void {
        this.snackBarRef.dismissWithAction();
    }

}
