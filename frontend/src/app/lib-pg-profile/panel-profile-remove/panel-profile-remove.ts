import { CommonModule } from "@angular/common";
import { ChangeDetectionStrategy, Component, EventEmitter, inject, Input, Output, ViewEncapsulation } from "@angular/core";
import { ReactiveFormsModule } from "@angular/forms";
import { MatButtonModule } from "@angular/material/button";
import { MatInputModule } from "@angular/material/input";
import { TranslatePipe, TranslateService } from "@ngx-translate/core";
import { DialogSrv } from "../../lib-dialog/dialog-srv";
import { ErrMsgObj } from "../../utils/http-error.util";

@Component({
    selector: 'app-panel-profile-remove',
    exportAs: "appPanelProfileRemove",
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatButtonModule, MatInputModule, TranslatePipe],
    templateUrl: './panel-profile-remove.html',
    styleUrl: './panel-profile-remove.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelProfileRemove {
    private dialogSrv: DialogSrv = inject(DialogSrv);
    private translateSrv: TranslateService = inject(TranslateService);

    @Input()
    public errMsgObjs: ErrMsgObj[] = [];
    @Input()
    public isDisabled: boolean | null | undefined;
    @Input()
    public nickname: string | null | undefined = "";

    @Output()
    readonly deleteAccount: EventEmitter<void> = new EventEmitter();

    // ** Public API **

    public removeAccount(): void {
        const title = this.translateSrv.instant("panel-profile-remove.dialog_title_question_account");
        const nickname = this.nickname;
        const appName = this.translateSrv.instant("app.name");
        const message = this.translateSrv.instant("panel-profile-remove.dialog_message_question_account", { nickname, appName: appName });
        const params = { btnNameCancel: "buttons.no", btnNameAccept: "buttons.yes" };
        this.dialogSrv.openConfirmation(message, title, params, { maxWidth: "40vw" })
            .then((respose) => {
                if (!!respose) {
                    this.deleteAccount.emit();
                }
            });
    }

    // ** Private API **
}
