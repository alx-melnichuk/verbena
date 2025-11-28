import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgBannedComponent } from './pg-banned.component';

describe('PgBannedComponent', () => {
  let component: PgBannedComponent;
  let fixture: ComponentFixture<PgBannedComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgBannedComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgBannedComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
