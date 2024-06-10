import { ChangeDetectionStrategy, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';

import { PanelProfileInfoComponent } from '../lib-profile/panel-profile-info/panel-profile-info.component';
import { UserDto } from '../lib-user/user-api.interface';
import { ActivatedRoute } from '@angular/router';
import { UserService } from '../lib-user/user.service';

@Component({
  selector: 'app-pg-profile',
  standalone: true,
  imports: [CommonModule, PanelProfileInfoComponent],
  templateUrl: './pg-profile.component.html',
  styleUrls: ['./pg-profile.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgProfileComponent {

  public userDto: UserDto;
  
  constructor(
    private route: ActivatedRoute,
    private userService: UserService,
  ) {
    this.userDto = this.route.snapshot.data['userDto'];
    console.log(`PgProfile() userDto=`, this.userDto); // #
  }
  
  // ** Public API **
  
  // ** Private API **

}
