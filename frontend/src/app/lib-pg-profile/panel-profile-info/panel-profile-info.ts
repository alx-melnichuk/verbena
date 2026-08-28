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
import { UniquenessDto } from "../../lib-user/user-dto";
import { UserSrv } from "../../lib-user/user-srv";
import { FileSizeUtil } from "../../utils/file_size.util";
import { ErrMsgObj } from "../../utils/http-error.util";
import { ValidFileTypesUtil } from "../../utils/valid_file_types.util";
import { ProfileConfigDto } from "../profile-config-dto";
import { ProfileDto, ModifyProfileDto, ProfileDtoUtil } from "../profile-dto";

export const PPI_DEBOUNCE_DELAY = 900;

@Component({
    selector: 'app-panel-profile-info',
    exportAs: "appPanelProfileInfo",
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatButtonModule, MatInputModule, TranslatePipe, UniquenessCheck,
        FieldColorTheme, FieldInput, FieldImage, FieldLocale, FieldTextarea],
    templateUrl: './panel-profile-info.html',
    styleUrl: './panel-profile-info.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelProfileInfo implements OnInit, OnChanges {
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private userSrv: UserSrv = inject(UserSrv);

    @Input()
    public errMsgObjs: ErrMsgObj[] = [];
    @Input()
    public isDisabledSubmit: boolean = false;
    @Input()
    public profileDto: ProfileDto | null = null;
    @Input()
    public profileConfigDto: ProfileConfigDto | null = null;

    @ViewChild("fieldNickname", { static: true })
    public fieldNicknameComp!: FieldInput;
    @ViewChild("fieldEmail", { static: true })
    public fieldEmailComp!: FieldInput;

    @Output()
    readonly updateProfile: EventEmitter<{ modifyProfile: ModifyProfileDto, avatarFile: File | null | undefined }> = new EventEmitter();

    public cntlsProfile = {
        avatar: new FormControl("", []),
        nickname: new FormControl("", []),
        email: new FormControl("", []),
        theme: new FormControl("", []),
        locale: new FormControl("", []),
        descript: new FormControl("", []),
    };
    public formGroupProfile: FormGroup = new FormGroup(this.cntlsProfile);

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

    private origProfileDto: ProfileDto = ProfileDtoUtil.create();

    ngOnInit(): void {
        this.cntlsProfile.nickname.markAsTouched();
        this.fieldNicknameComp.markAsTouched();
        this.cntlsProfile.email.markAsTouched();
        this.fieldEmailComp.markAsTouched();
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["profileDto"]) {
            this.prepareFormGroupByProfileDto(this.profileDto);
        }
        if (!!changes["profileConfigDto"]) {
            this.prepareFormGroupByProfileConfigDto(this.profileConfigDto);
        }
        if (!!changes["isDisabledSubmit"]) {
            if (this.isDisabledSubmit != this.formGroupProfile.disabled) {
                this.isDisabledSubmit ? this.formGroupProfile.disable() : this.formGroupProfile.enable();
                this.changeDetector.markForCheck();
            }
        }
    }

    // ** Public API **

    // ** Section: Update profile (formGroupProfile) **

    public checkUniqueNickname = (nickname: string | null | undefined): Promise<boolean> => {
        if (!nickname || this.origProfileDto.nickname.toLowerCase() == nickname.toLowerCase()) {
            return Promise.resolve(true);
        }
        // #Перенести проверку на уровень выше.
        return this.userSrv.uniqueness(nickname, "").then((response) => response == null || (response as UniquenessDto).uniqueness);
    }

    public checkUniqueEmail = (email: string | null | undefined): Promise<boolean> => {
        if (!email || this.origProfileDto.email.toLowerCase() == email.toLowerCase()) {
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
            this.cntlsProfile.avatar.markAsDirty();
        } else {
            this.cntlsProfile.avatar.markAsPristine();
        }
    }

    public updateErrMsgObjs(errMsgObjs: ErrMsgObj[] = []): void {
        this.errMsgObjs = errMsgObjs;
    }

    public saveProfile(formGroup: FormGroup): void {
        if (!formGroup || formGroup.pristine || formGroup.invalid) {
            return;
        }
        const nickname = formGroup.get("nickname")?.value || "";
        const email = formGroup.get("email")?.value || "";
        const descript = formGroup.get("descript")?.value;
        const theme = formGroup.get("theme")?.value;
        const locale = formGroup.get("locale")?.value;

        const modifyProfile: ModifyProfileDto = {
            nickname: (this.origProfileDto.nickname != nickname ? nickname : undefined),
            email: (this.origProfileDto.email != email ? email : undefined),
            descript: (this.origProfileDto.descript != descript ? descript : undefined),
            theme: (this.origProfileDto.theme != theme ? theme : undefined),
            locale: (this.origProfileDto.locale != locale ? locale : undefined),
        };
        const is_all_empty = Object.values(modifyProfile).findIndex((value) => value !== undefined) == -1;
        if (!is_all_empty || this.avatarFile !== undefined) {
            this.updateProfile.emit({ modifyProfile, avatarFile: this.avatarFile });
        }
    }

    // ** Private API **

    private prepareFormGroupByProfileDto(profileDto: ProfileDto | null): void {
        if (!profileDto) {
            return;
        }
        this.origProfileDto = ProfileDtoUtil.create(profileDto);
        this.origProfileDto.descript = (profileDto.descript || "");
        this.origProfileDto.theme = (profileDto.theme || "");
        this.origProfileDto.locale = (profileDto.locale || "");

        Object.freeze(this.origProfileDto);

        this.formGroupProfile.patchValue({
            nickname: profileDto.nickname,
            email: profileDto.email,
            descript: (profileDto.descript || ""),
            theme: (profileDto.theme || ""),
            locale: (profileDto.locale || ""),
            avatar: profileDto.avatar,
        });
        this.avatarFile = undefined;
        this.initIsAvatar = !!profileDto.avatar;
        this.formGroupProfile.markAsPristine();
    }

    private prepareFormGroupByProfileConfigDto(profileConfigDto: ProfileConfigDto | null): void {
        // Set FieldImage parameters
        this.maxSize = profileConfigDto?.avatarMaxSize || MAX_FILE_SIZE;
        this.accepts = (profileConfigDto?.avatarValidTypes || []).join(",");
        this.availableFileTypes = ValidFileTypesUtil.text(this.accepts).join(", ").toUpperCase();
        this.availableMaxFileSize = FileSizeUtil.formatBytes(this.maxSize, 1);
    }
}
