import { CommonModule } from "@angular/common";
import { ChangeDetectionStrategy, Component, inject, Inject, OnInit, ViewEncapsulation } from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import { MatDialogModule, MatDialogRef, MAT_DIALOG_DATA } from "@angular/material/dialog";
import { TranslateService } from "@ngx-translate/core";

export interface ConfirmationData {
    title?: string;
    message?: string;
    messageHtml?: string;
    btnNameCancel?: string;
    btnNameAccept?: string;
}

@Component({
    selector: "app-confirmation",
    exportAs: "appConfirmation",
    standalone: true,
    imports: [CommonModule, MatDialogModule, MatButtonModule],
    templateUrl: "./confirmation.html",
    styleUrl: "./confirmation.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class Confirmation implements OnInit {
    readonly dialogRef: MatDialogRef<Confirmation> = inject(MatDialogRef<Confirmation>);
    readonly data: ConfirmationData = inject<ConfirmationData>(MAT_DIALOG_DATA);

    private translate: TranslateService = inject(TranslateService);

    public title = "dialog.confirmation";
    public message: string | null = null;
    public messageHtml: string | null = null;
    public btnNameCancel: string | null = "Cancel";
    public btnNameAccept: string | null = "Accept";

    constructor() {
        this.title = this.getTranslate(this.data.title || this.title);
        this.message = this.getTranslate(this.data.message || this.message);
        this.messageHtml = this.getTranslate(this.data.messageHtml || this.messageHtml);
        this.btnNameCancel = this.data.btnNameCancel != null ? this.getTranslate(this.data.btnNameCancel || this.btnNameCancel) : null;
        this.btnNameAccept = this.data.btnNameAccept != null ? this.getTranslate(this.data.btnNameAccept || this.btnNameAccept) : null;
    }

    ngOnInit(): void { }

    public cancel(): void {
        this.dialogRef.close();
    }

    public accept(): void {
        this.dialogRef.close(true);
    }

    private getTranslate(name: string | null): string {
        return !!name ? this.translate.instant(name) : (name || "");
    }
}
