import { CommonModule } from "@angular/common";
import {
    ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, inject, Input, OnChanges, OnInit, Output, SimpleChanges, ViewChild, ViewEncapsulation
} from "@angular/core";
import { FormControl, FormGroup, ReactiveFormsModule } from "@angular/forms";
import { MatButtonModule } from "@angular/material/button";
import { MatInputModule } from "@angular/material/input";
import { TranslatePipe } from "@ngx-translate/core";
import { CN_NICKNAME, CN_EMAIL, CN_DESCRIPT } from "../../common/fields-consts";
import { IMAGE_VALID_FILE_TYPES, MAX_FILE_SIZE } from "../../common/file-upload-consts";
import { FieldColorTheme } from "../../components/field-color-theme/field-color-theme";
import { FieldImage } from "../../components/field-image/field-image";
import { FieldInput } from "../../components/field-input/field-input";
import { FieldLocale } from "../../components/field-locale/field-locale";
import { FieldTextarea } from "../../components/field-textarea/field-textarea";
import { UniquenessCheck } from "../../components/uniqueness-check/uniqueness-check";
import { UniquenessDto, UserDto } from "../../lib-user/user-dto";
import { UserSrv } from "../../lib-user/user-srv";
import { FileSizeUtil } from "../../utils/file_size.util";
import { ErrMsgObj } from "../../utils/http-error.util";
import { ValidFileTypesUtil } from "../../utils/valid_file_types.util";
import { ProfileConfigDto } from "../profile-config-dto";
import { ModifyProfileDto } from "../profile-dto";

export const PPI_DEBOUNCE_DELAY = 900;

@Component({
    selector: "app-panel-profile-info",
    exportAs: "appPanelProfileInfo",
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatButtonModule, MatInputModule, TranslatePipe, UniquenessCheck,
        FieldColorTheme, FieldInput, FieldImage, FieldLocale, FieldTextarea],
    templateUrl: "./panel-profile-info.html",
    styleUrl: "./panel-profile-info.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelProfileInfo implements OnInit, OnChanges {
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private userSrv: UserSrv = inject(UserSrv);

    @Input()
    public errMsgObjs: ErrMsgObj[] = [];
    @Input()
    public isDisabled: boolean | null | undefined;
    @Input()
    public profileDto: UserDto | null | undefined;
    @Input()
    public profileConfigDto: ProfileConfigDto | null | undefined;

    @ViewChild("fieldNickname", { static: true })
    public fieldNicknameComp!: FieldInput;
    @ViewChild("fieldEmail", { static: true })
    public fieldEmailComp!: FieldInput;

    @Output()
    readonly changeData: EventEmitter<boolean> = new EventEmitter();
    @Output()
    readonly updateProfile: EventEmitter<{ modifyProfile: ModifyProfileDto, avatarFile: File | null | undefined }> = new EventEmitter();

    public cntls = {
        avatar: new FormControl("", []),
        nickname: new FormControl("", []),
        email: new FormControl("", []),
        theme: new FormControl("", []),
        locale: new FormControl("", []),
        descript: new FormControl("", []),
    };
    public formGroup: FormGroup = new FormGroup(this.cntls);

    public cn_nickname = CN_NICKNAME;
    public cn_email = CN_EMAIL;
    public cn_descript = CN_DESCRIPT;

    public debounceDelay: number = PPI_DEBOUNCE_DELAY;

    // FieldImage parameters
    public accepts = IMAGE_VALID_FILE_TYPES;
    public maxSize = MAX_FILE_SIZE;
    public availableFileTypes: string = "";
    public availableMaxFileSize: string = "";

    // FieldImage FormControl
    public avatarFile: File | null | undefined;
    public initIsAvatar: boolean = false; // original has an avatar.

    private isChangeData: boolean = false;

    ngOnInit(): void {
        this.cntls.nickname.markAsTouched();
        this.fieldNicknameComp.markAsTouched();
        this.cntls.email.markAsTouched();
        this.fieldEmailComp.markAsTouched();
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["profileDto"]) {
            const profileInfo = {
                avatar: this.profileDto?.avatar || "",
                nickname: this.profileDto?.nickname || "",
                email: this.profileDto?.email || "",
                theme: this.profileDto?.theme || "",
                locale: this.profileDto?.locale || "",
                descript: this.profileDto?.descript || "",
            };
            this.formGroup.patchValue(profileInfo);
            this.avatarFile = undefined;
            this.initIsAvatar = !!profileInfo.avatar;
            this.formGroup.markAsPristine();
        }
        if (!!changes["profileConfigDto"]) {
            // Set FieldImage parameters
            this.maxSize = this.profileConfigDto?.avatarMaxSize || MAX_FILE_SIZE;
            this.accepts = (this.profileConfigDto?.avatarValidTypes || []).join(",");
            this.availableFileTypes = ValidFileTypesUtil.text(this.accepts).join(", ").toUpperCase();
            this.availableMaxFileSize = FileSizeUtil.formatBytes(this.maxSize, 1);

        }
        if (!!changes["isDisabled"]) {
            if (!!this.isDisabled != this.formGroup.disabled) {
                !!this.isDisabled ? this.formGroup.disable() : this.formGroup.enable();
                this.changeDetector.markForCheck();
            }
        }
    }

    // ** Public API **

    public doChangeForm(): void {
        const isDirty = this.formGroup.dirty;
        const isValid = this.formGroup.valid;
        console.log(`doChangeForm() isDirty: ${isDirty}, isValid: ${isValid}`); // #

    }
    public doChangeData(name: string, formGroup: FormGroup, origValues: unknown): void {
        const origValues2 = (origValues as Record<string, string>) || {};
        if (!name || !formGroup) {
            return;
        }
        if ((formGroup.controls[name]?.value || "") == (origValues2[name] || "")) {
            formGroup.controls[name].markAsPristine();
        }
        if (this.isChangeData != formGroup.dirty) {
            this.changeData.emit(this.isChangeData = formGroup.dirty);
        }
    }

    // ** Section: Update profile (formGroup) **

    public checkUniqueNickname = (nickname: string | null | undefined): Promise<boolean> => {
        if (!nickname || this.profileDto?.nickname.toLowerCase() == nickname.toLowerCase()) {
            return Promise.resolve(true);
        }
        // #Перенести проверку на уровень выше.
        return this.userSrv.uniqueness(nickname, "").then((response) => response == null || (response as UniquenessDto).uniqueness);
    }

    public checkUniqueEmail = (email: string | null | undefined): Promise<boolean> => {
        if (!email || this.profileDto?.email.toLowerCase() == email.toLowerCase()) {
            return Promise.resolve(true);
        }
        // #Перенести проверку на уровень выше.
        return this.userSrv.uniqueness("", email).then((response) => response == null || (response as UniquenessDto).uniqueness);
    }

    public addAvatarFile(file: File): void {
        this.avatarFile = file;
    }

    public deleteAvatarFile(): void {
        this.avatarFile = (!!this.initIsAvatar ? null : undefined);
        if (!!this.initIsAvatar) {
            this.cntls.avatar.markAsDirty();
        } else {
            this.cntls.avatar.markAsPristine();
        }
    }

    public updateErrMsgObjs(errMsgObjs: ErrMsgObj[] = []): void {
        this.errMsgObjs = errMsgObjs;
    }

    public saveProfile(formGroup: FormGroup, profileDto: UserDto | null | undefined): void {
        if (!formGroup || formGroup.pristine || formGroup.invalid) {
            return;
        }
        const nickname = formGroup.get("nickname")?.value || "";
        const email = formGroup.get("email")?.value || "";
        const descript = formGroup.get("descript")?.value;
        const theme = formGroup.get("theme")?.value;
        const locale = formGroup.get("locale")?.value;

        const modifyProfile: ModifyProfileDto = {
            nickname: (profileDto?.nickname != nickname ? nickname : undefined),
            email: (profileDto?.email != email ? email : undefined),
            descript: (profileDto?.descript != descript ? descript : undefined),
            theme: (profileDto?.theme != theme ? theme : undefined),
            locale: (profileDto?.locale != locale ? locale : undefined),
        };
        const is_all_empty = Object.values(modifyProfile).findIndex((value) => value !== undefined) == -1;
        if (!is_all_empty || this.avatarFile !== undefined) {
            this.updateProfile.emit({ modifyProfile, avatarFile: this.avatarFile });
        }
    }

    // ** Private API **
}
