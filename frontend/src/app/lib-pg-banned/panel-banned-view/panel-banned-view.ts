import { CommonModule } from "@angular/common";
import { ChangeDetectionStrategy, Component, EventEmitter, inject, Input, Output, ViewEncapsulation } from "@angular/core";
import { PanelBannedUsers } from "../panel-banned-users/panel-banned-users";
import { BlockedUserDto } from "../../lib-chat/chat-message";
import { TranslateService } from "@ngx-translate/core";
import { LocaleSrv } from "../../common/locale-srv";
import { DialogSrv } from "../../lib-dialog/dialog-srv";

@Component({
    selector: "app-panel-banned-view",
    exportAs: "appPanelBannedView",
    standalone: true,
    imports: [CommonModule, PanelBannedUsers],
    templateUrl: "./panel-banned-view.html",
    styleUrl: "./panel-banned-view.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelBannedView {
    public localeSrv: LocaleSrv = inject(LocaleSrv);
    private dialogSrv: DialogSrv = inject(DialogSrv);
    private translateSrv: TranslateService = inject(TranslateService);

    @Input()
    public blockedUsers: BlockedUserDto[] = [];
    @Input()
    public isLoading: boolean | null = null;
    @Input()
    public sortColumn: string | undefined | null;
    @Input()
    public sortDesc: boolean | undefined | null;

    @Output()
    readonly sort: EventEmitter<Record<string, boolean>> = new EventEmitter();
    @Output()
    readonly unblockUser: EventEmitter<string> = new EventEmitter();

    public title: string | null = "panel-banned-users.title";

    // ** Public API **

    public doSort(event: Record<string, boolean>): void {
        this.sort.emit(event);
    }

    public doUnblockUser(nickname: string): void {
        if (!nickname) {
            return;
        }
        const message = this.translateSrv.instant("panel-banned-view.are_you_want_to_unblock_user", { nickname });
        this.dialogSrv.openConfirmation(message, "", { btnNameCancel: "buttons.no", btnNameAccept: "buttons.yes" })
            .then((res) => {
                if (!!res) {
                    this.unblockUser.emit(nickname);
                }
            });
    }
}
