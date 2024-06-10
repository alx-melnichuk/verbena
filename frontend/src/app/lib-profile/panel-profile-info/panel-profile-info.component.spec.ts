import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelProfileInfoComponent } from './panel-profile-info.component';

describe('PanelProfileInfoComponent', () => {
  let component: PanelProfileInfoComponent;
  let fixture: ComponentFixture<PanelProfileInfoComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PanelProfileInfoComponent]
    });
    fixture = TestBed.createComponent(PanelProfileInfoComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
