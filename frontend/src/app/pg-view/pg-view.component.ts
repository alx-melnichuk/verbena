import { ChangeDetectionStrategy, Component, OnInit, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { UserService } from '../entities/user/user.service';

@Component({
  selector: 'app-pg-view',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './pg-view.component.html',
  styleUrls: ['./pg-view.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PgViewComponent implements OnInit {
  constructor(private userService: UserService) {
    console.log(`PgViewComponent()`); // #-
  }
  ngOnInit(): void {
    this.userService.getCurrentUser().then((res) => {
      console.log(`^^^ res:`, res);
    });
  }
}
