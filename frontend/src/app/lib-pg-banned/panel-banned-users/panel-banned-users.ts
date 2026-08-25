import { CommonModule } from "@angular/common";
import { ChangeDetectionStrategy, Component, EventEmitter, Input, Output, ViewEncapsulation } from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import { TranslatePipe } from "@ngx-translate/core";
import { DateTimeFormatPipe } from "../../common/date-time-format-pipe";
import { Image } from "../../components/image/image";
import { BlockedUserDto } from "../../lib-chat/chat-message";

const COL_NICKNAME = "nickname";
const COL_EMAIL = "email";
const COL_BLOCK_DATE = "block_date";

@Component({
    selector: "app-panel-banned-users",
    exportAs: "appPanelBannedUsers",
    standalone: true,
    imports: [CommonModule, MatButtonModule, TranslatePipe, TranslatePipe, Image, DateTimeFormatPipe],
    templateUrl: "./panel-banned-users.html",
    styleUrl: "./panel-banned-users.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelBannedUsers {
    @Input()
    public blockedUsers: BlockedUserDto[] = [];
    @Input()
    public isLoading: boolean | null = null;
    @Input()
    public locale: string | null | undefined;
    @Input()
    public title: string | null = "panel-banned-users.title";
    @Input()
    public sortColumn: string | undefined | null;
    @Input()
    public sortDesc: boolean | undefined | null;

    @Output()
    readonly sort: EventEmitter<Record<string, boolean>> = new EventEmitter();
    @Output()
    readonly unblockUser: EventEmitter<string> = new EventEmitter();

    readonly formatDate: Intl.DateTimeFormatOptions = { dateStyle: "medium" };
    readonly formatTime: Intl.DateTimeFormatOptions = { timeStyle: "short" };
    readonly colNickname: string = COL_NICKNAME;
    readonly colEmail: string = COL_EMAIL;
    readonly colBlockDate: string = COL_BLOCK_DATE;

    // ** Public API **

    public doSort(newColumn: string, sortColumn: string | undefined | null, sortDesc: boolean): void {
        if (!!newColumn) {
            const newDesc = newColumn == sortColumn ? !sortDesc : false;
            this.sort.emit({ [newColumn]: newDesc });
        }
    }

    public doUnblockUser(nickname: string): void {
        if (!!nickname) {
            this.unblockUser.emit(nickname);
        }
    }
}
