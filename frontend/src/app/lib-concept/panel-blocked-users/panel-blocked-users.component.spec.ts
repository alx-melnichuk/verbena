import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelBlockedUsersComponent } from './panel-blocked-users.component';

describe('PanelBlockedUsersComponent', () => {
  let component: PanelBlockedUsersComponent;
  let fixture: ComponentFixture<PanelBlockedUsersComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelBlockedUsersComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelBlockedUsersComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
