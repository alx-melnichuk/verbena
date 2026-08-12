import { CommonModule, KeyValue } from "@angular/common";
import {
    Component, ViewEncapsulation, ChangeDetectionStrategy, AfterContentInit, Input, Output, EventEmitter, inject, ChangeDetectorRef,
    OnChanges, SimpleChanges
} from "@angular/core";
import { TranslatePipe, TranslateService } from "@ngx-translate/core";
import { LocaleSrv } from "../../common/locale-srv";
import { StringDateTime } from "../../common/string-date-time";
import { Sidebar } from "../../components/sidebar/sidebar";
import { Spinner } from "../../components/spinner/spinner";
import { ChatMessageDto } from "../../lib-chat/chat-message";
import { PanelChat } from "../../lib-chat/panel-chat/panel-chat";
import { DialogSrv } from "../../lib-dialog/dialog-srv";
import { StreamDto, StreamState } from "../../lib-stream/stream-dto";
import { StringUtil } from "../../utils/string.util";
import { PanelStreamActions } from "../panel-stream-actions/panel-stream-actions";
import { PanelStreamParams } from "../panel-stream-params/panel-stream-params";
import { PanelStreamState } from "../panel-stream-state/panel-stream-state";
import { ItemViewSetMsg } from "../pg-browse-view/pg-browse-view";

@Component({
    selector: "app-panel-browse-view",
    exportAs: "appPanelBrowseView",
    standalone: true,
    imports: [CommonModule, Spinner, Sidebar, TranslatePipe,
        PanelStreamState, PanelStreamParams, PanelStreamActions, PanelChat],
    templateUrl: "./panel-browse-view.html",
    styleUrl: "./panel-browse-view.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelBrowseView implements AfterContentInit, OnChanges {
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private dialogSrv: DialogSrv = inject(DialogSrv);
    public localeSrv: LocaleSrv = inject(LocaleSrv);
    private translateSrv: TranslateService = inject(TranslateService);

    @Input()
    public chatBlockedUsers: string[] = []; // List of new blocked users.
    @Input()
    public chatDeleteIds: number[] = []; // A list of message IDs that have been deleted from the chat.
    @Input()
    public chatIsBlocked: boolean | null | undefined; // Indication that the user is blocked.
    @Input()
    public chatIsEditable: boolean | null | undefined; // Indicates that the user can send messages to the chat.
    @Input()
    public chatIsLoading: boolean | null | undefined; // Indicates that data is being loaded.
    @Input()
    public chatIsOwner: boolean | null | undefined; // Indicates that the user is the owner of the chat.
    @Input()
    public chatIsReset: boolean | null | undefined; // Checked only together with "itemPage".
    @Input()
    public chatItemSetMsg: ItemViewSetMsg | null | undefined;
    @Input()
    public chatMaxLen: number | null | undefined;
    @Input()
    public chatMinLen: number | null | undefined;
    @Input()
    public chatMaxRows: number | null | undefined;
    @Input()
    public chatMinRows: number | null | undefined;
    @Input()
    public chatMaxSizeRows: number | null | undefined; // The maximum number of rows in the buffer.
    @Input()
    public chatNewMsg: ChatMessageDto | null | undefined;
    @Input()
    public chatNickname: string | null | undefined;

    @Input()
    public countOfViewer: number | null | undefined;

    @Input()
    public isLoadStream = false;
    @Input()
    public isStreamOwner: boolean = false;
    @Input()
    public nickname: string | null | undefined;

    @Input()
    public ownerAvatar: string | null | undefined;
    @Input()
    public ownerEmail: string | null | undefined;
    @Input()
    public ownerNickname: string | null | undefined;

    @Input()
    public streamDto: StreamDto | null = null;

    @Input()
    public timerActive: boolean | null | undefined;
    @Input()
    public timerIsShow: boolean | null | undefined;
    @Input()
    public timerValue: number | null | undefined;

    @Output()
    readonly changeState: EventEmitter<StreamState> = new EventEmitter();
    @Output()
    readonly blockUser: EventEmitter<string> = new EventEmitter();
    @Output()
    readonly unblockUser: EventEmitter<string> = new EventEmitter();
    @Output()
    readonly sendMessage: EventEmitter<string> = new EventEmitter();
    @Output()
    readonly editMessage: EventEmitter<KeyValue<number, string>> = new EventEmitter();
    @Output()
    readonly cutMessage: EventEmitter<number> = new EventEmitter();
    @Output()
    readonly rmvMessage: EventEmitter<number> = new EventEmitter();
    @Output()
    readonly loadPage: EventEmitter<{ page: number, limit: number, mode: number, isNext: boolean, date: string }> = new EventEmitter();
    @Output()
    readonly loadSet: EventEmitter<{ isAddTop: boolean, date: StringDateTime | undefined, count: number }> = new EventEmitter();

    public isSidebarLfOpen: boolean = false;
    public isSidebarRgOpen: boolean = false;
    // To disable the jumping effect of the "stream-video" panel at startup.
    public isStreamVideo = false;
    public ownerNameSymbols: string = "";

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["ownerNickname"]) {
            this.ownerNameSymbols = (StringUtil.capitalizeOnlyFirstLetter(this.ownerNickname) || "").slice(0, 2);
        }
    }

    // To disable the jumping effect of the "stream-video" panel at startup.
    ngAfterContentInit(): void {
        this.isStreamVideo = true;
        this.changeDetector.markForCheck();
    }

    // ** Public API **

    // ** Side Left and Right **

    public clearEvent(event: Event): void {
        event.preventDefault();
        event.stopPropagation();
    }

    // Section: "panel stream admin"

    public doChangeState(newState: StreamState | undefined): void {
        this.changeState.emit(newState);
    }

    // Section: "Chat"

    public doBlockUser(user_name: string): void {
        if (!!user_name) {
            this.blockUser.emit(user_name);
        }
    }
    public doUnblockUser(user_name: string): void {
        if (!!user_name) {
            this.unblockUser.emit(user_name);
        }
    }
    public doSendMessage(newMessage: string): void {
        if (!!newMessage) {
            this.sendMessage.emit(newMessage);
        }
    }
    public doEditMessage(keyValue: KeyValue<number, string>): void {
        if (!!keyValue && !!keyValue.key) {
            this.editMessage.emit(keyValue);
        }
    }
    public doCutMessage(keyValue: KeyValue<number, string>): void {
        if (!keyValue || !keyValue.key || !keyValue.value) {
            return;
        }
        const msg = keyValue.value.slice(0, 45) + (keyValue.value.length > 45 ? "..." : "");
        const message = this.translateSrv.instant("panel-browse-view.sure_you_want_delete_message", { message: msg });
        this.dialogSrv.openConfirmation(message, "", { btnNameCancel: "buttons.no", btnNameAccept: "buttons.yes" })
            .then((res) => {
                if (!!res) {
                    this.cutMessage.emit(keyValue.key);
                }
            });

    }
    public doRmvMessage(chMsgId: number): void {
        if (!chMsgId || chMsgId < 0) {
            return;
        }
        const message = this.translateSrv.instant("panel-browse-view.sure_you_want_permanently_delete_message");
        this.dialogSrv.openConfirmation(message, "", { btnNameCancel: "buttons.no", btnNameAccept: "buttons.yes" })
            .then((res) => {
                if (!!res) {
                    this.rmvMessage.emit(chMsgId);
                }
            });
    }
    public doLoadSet(isAddTop: boolean, date: StringDateTime | undefined, count: number): void {
        this.loadSet.emit({ isAddTop, date, count });
    }

    // ** Private API **
}
