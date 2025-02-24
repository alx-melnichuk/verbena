import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgProfileComponent } from './pg-profile.component';

describe('PgProfileComponent', () => {
  let component: PgProfileComponent;
  let fixture: ComponentFixture<PgProfileComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PgProfileComponent]
    });
    fixture = TestBed.createComponent(PgProfileComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
